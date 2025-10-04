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
            if let Some(dup) = self.duplication.take() {
                unsafe {
                    let _ = dup.ReleaseFrame();
                }
            }
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

                // Try to acquire next frame
                match duplication.AcquireNextFrame(
                    0, // Timeout in ms (0 = no wait)
                    &mut frame_info,
                    &mut desktop_resource,
                ) {
                    Ok(_) => {
                        // Frame acquired successfully
                        let resource = desktop_resource.unwrap();

                        // Convert to texture
                        let texture: ID3D11Texture2D = resource.cast()
                            .context("Failed to cast to texture")?;

                        // Create staging texture for CPU readback
                        let mut desc = D3D11_TEXTURE2D_DESC::default();
                        texture.GetDesc(&mut desc);

                        desc.Usage = D3D11_USAGE_STAGING;
                        desc.BindFlags = 0;
                        desc.CPUAccessFlags = 1; // D3D11_CPU_ACCESS_READ
                        desc.MiscFlags = 0;

                        let staging_texture = {
                            let mut texture = None;
                            self.device.CreateTexture2D(&desc, None, Some(&mut texture))?;
                            texture.context("Failed to create staging texture")?
                        };

                        // Copy from GPU to staging
                        self.context.CopyResource(&staging_texture, &texture);

                        // Map staging texture to read pixels
                        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                        self.context.Map(
                            &staging_texture,
                            0,
                            D3D11_MAP_READ,
                            0,
                            Some(&mut mapped),
                        ).context("Failed to map texture")?;

                        // Copy pixel data
                        let data_size = (mapped.RowPitch * desc.Height) as usize;
                        let src_ptr = mapped.pData as *const u8;
                        let data = std::slice::from_raw_parts(src_ptr, data_size).to_vec();

                        // Unmap
                        self.context.Unmap(&staging_texture, 0);

                        // Release frame
                        duplication.ReleaseFrame().ok();

                        // Update dimensions if needed (after releasing borrow)
                        if self.width == 0 || self.height == 0 {
                            self.width = desc.Width;
                            self.height = desc.Height;
                            tracing::info!("Capture dimensions: {}x{}", self.width, self.height);
                        }

                        let frame = CapturedFrame::new(
                            data,
                            desc.Width,
                            desc.Height,
                            mapped.RowPitch,
                        );

                        Ok(Some(frame))
                    }
                    Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => {
                        // No new frame available
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
            let _ = self.stop();
        }
    }
}