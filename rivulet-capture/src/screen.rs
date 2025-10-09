#[cfg(windows)]
pub mod windows {
    use anyhow::{Context, Result};
    use windows::{
        core::*,
        Win32::Graphics::{
            Direct3D::D3D_DRIVER_TYPE,
            Direct3D11::*,
            Dxgi::{Common::*, *},
        },
        Win32::Foundation::*,
    };
    use crate::{CaptureSource, CapturedFrame};

    pub struct DxgiScreenCapture {
        device: ID3D11Device,
        context: ID3D11DeviceContext,
        duplication: Option<IDXGIOutputDuplication>,
        width: u32,
        height: u32,
        capturing: bool,
    }

    impl DxgiScreenCapture {
        pub fn new(display_index: u32) -> Result<Self> {
            tracing::info!("Initializing DXGI screen capture for display {}", display_index);

            unsafe {
                // Create D3D11 Device
                let mut device = None;
                let mut context = None;

                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE(1), // HARDWARE
                    None,
                    D3D11_CREATE_DEVICE_FLAG(0),
                    None,
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    None,
                    Some(&mut context),
                ).context("Failed to create D3D11 device")?;

                let device = device.unwrap();
                let context = context.unwrap();

                tracing::info!("D3D11 device created successfully");

                Ok(Self {
                    device,
                    context,
                    duplication: None,
                    width: 0,
                    height: 0,
                    capturing: false,
                })
            }
        }

        fn init_duplication(&mut self) -> Result<()> {
            if self.duplication.is_some() {
                return Ok(());
            }

            unsafe {
                let dxgi_device: IDXGIDevice = self.device.cast()?;
                let adapter = dxgi_device.GetAdapter()?;
                let output = adapter.EnumOutputs(0)?;
                let output1: IDXGIOutput1 = output.cast()?;

                let duplication = output1.DuplicateOutput(&self.device)
                    .context("Failed to duplicate output - another capture may be active")?;

                self.duplication = Some(duplication);
                tracing::info!("Desktop duplication initialized");
                Ok(())
            }
        }
    }

    impl CaptureSource for DxgiScreenCapture {
        fn start(&mut self) -> Result<()> {
            tracing::info!("Starting screen capture");
            self.init_duplication()?;
            self.capturing = true;
            Ok(())
        }

        fn stop(&mut self) -> Result<()> {
            tracing::info!("Stopping screen capture");
            self.capturing = false;
            self.duplication = None;
            Ok(())
        }

        fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
            if !self.capturing {
                return Ok(None);
            }

            unsafe {
                let duplication = match &self.duplication {
                    Some(dup) => dup,
                    None => return Err(anyhow::anyhow!("Duplication not initialized")),
                };

                let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
                let mut desktop_resource = None;

                tracing::debug!("Attempting to acquire frame...");

                // Try to acquire next frame
                let acquire_result = duplication.AcquireNextFrame(
                    500, // 500ms timeout
                    &mut frame_info,
                    &mut desktop_resource,
                );

                tracing::debug!("AcquireNextFrame result: {:?}", acquire_result);

                match acquire_result {
                    Ok(_) => {
                        tracing::debug!("Frame acquired, processing...");

                        let result = (|| -> Result<CapturedFrame> {
                            tracing::debug!("Getting resource...");
                            let resource = desktop_resource.ok_or_else(|| anyhow::anyhow!("No resource"))?;

                            tracing::debug!("Casting to texture...");
                            let texture: ID3D11Texture2D = resource.cast()
                                .context("Failed to cast to texture")?;

                            tracing::debug!("Getting texture description...");
                            let mut desc = D3D11_TEXTURE2D_DESC::default();
                            texture.GetDesc(&mut desc);

                            tracing::debug!("Texture: {}x{}, Format: {:?}", desc.Width, desc.Height, desc.Format);

                            // Copy fields from original desc
                            let staging_desc = D3D11_TEXTURE2D_DESC {
                                Width: desc.Width,
                                Height: desc.Height,
                                MipLevels: desc.MipLevels,
                                ArraySize: desc.ArraySize,
                                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                                SampleDesc: desc.SampleDesc,
                                Usage: D3D11_USAGE_STAGING,
                                BindFlags: 0,
                                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                                MiscFlags: 0,
                            };

                            tracing::debug!("Creating staging texture with BGRA8 format...");
                            let staging_texture = {
                                let mut texture = None;
                                self.device.CreateTexture2D(&staging_desc, None, Some(&mut texture))
                                    .context("CreateTexture2D failed")?;
                                texture.context("Failed to create staging texture")?
                            };

                            // Since formats differ, we need an intermediate copy texture
                            // Create a render target texture with BGRA8 format
                            let mut copy_desc = staging_desc;
                            copy_desc.Usage = D3D11_USAGE_DEFAULT;
                            copy_desc.BindFlags = D3D11_BIND_RENDER_TARGET.0 as u32;
                            copy_desc.CPUAccessFlags = 0;

                            tracing::debug!("Creating intermediate copy texture...");
                            let copy_texture = {
                                let mut texture = None;
                                self.device.CreateTexture2D(&copy_desc, None, Some(&mut texture))
                                    .context("CreateTexture2D for copy failed")?;
                                texture.context("Failed to create copy texture")?
                            };

                            // Copy original -> copy_texture (GPU will handle format conversion)
                            tracing::debug!("Copying with format conversion...");
                            self.context.CopyResource(&copy_texture, &texture);

                            // Copy copy_texture -> staging (now same format)
                            self.context.CopyResource(&staging_texture, &copy_texture);

                            tracing::debug!("Mapping texture...");
                            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                            self.context.Map(
                                &staging_texture,
                                0,
                                D3D11_MAP_READ,
                                0,
                                Some(&mut mapped),
                            ).context("Failed to map texture")?;

                            tracing::debug!("Mapped: RowPitch={}", mapped.RowPitch);

                            // Copy pixel data
                            let data_size = (mapped.RowPitch * staging_desc.Height) as usize;
                            tracing::debug!("Copying {} bytes...", data_size);
                            let src_ptr = mapped.pData as *const u8;
                            let data = std::slice::from_raw_parts(src_ptr, data_size).to_vec();

                            tracing::debug!("Unmapping...");
                            self.context.Unmap(&staging_texture, 0);

                            // Update dimensions if needed
                            if self.width == 0 || self.height == 0 {
                                self.width = staging_desc.Width;
                                self.height = staging_desc.Height;
                                tracing::info!("Capture dimensions: {}x{}", self.width, self.height);
                            }

                            tracing::debug!("Creating frame...");
                            let frame = CapturedFrame::new(
                                data,
                                staging_desc.Width,
                                staging_desc.Height,
                                mapped.RowPitch,
                            );

                            Ok(frame)
                        })();

                        tracing::debug!("Frame processing result: {:?}", result.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));

                        // ALWAYS release frame after acquire, even on error
                        tracing::debug!("Releasing frame...");
                        let release_result = duplication.ReleaseFrame();
                        tracing::debug!("ReleaseFrame result: {:?}", release_result);
                        release_result.context("Failed to release frame")?;

                        result.map(Some)
                    }
                    Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => {
                        // No new frame available - this is normal
                        Ok(None)
                    }
                    Err(e) if e.code() == DXGI_ERROR_ACCESS_LOST => {
                        // Desktop duplication lost, need to recreate
                        tracing::warn!("Desktop duplication access lost, reinitializing");
                        self.duplication = None;
                        self.init_duplication()?;
                        Ok(None)
                    }
                    Err(e) => {
                        tracing::error!("AcquireNextFrame failed: {:?}", e);
                        Err(e.into())
                    }
                }
            }
        }

        fn dimensions(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn is_capturing(&self) -> bool {
            self.capturing
        }
    }

    impl Drop for DxgiScreenCapture {
        fn drop(&mut self) {
            self.capturing = false;
            self.duplication = None;
        }
    }
}