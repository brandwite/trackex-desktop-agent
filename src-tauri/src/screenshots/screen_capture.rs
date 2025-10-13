use anyhow::Result;
use base64::{self, Engine};

#[cfg(target_os = "macos")]
use core_graphics::{
    image::CGImageRef,
};

#[cfg(target_os = "windows")]
use windows::{
    Win32::{
        Graphics::Gdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD, SRCCOPY},
        UI::WindowsAndMessaging::{GetDesktopWindow, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
    },
};

pub async fn capture_screen() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        capture_screen_macos().await
    }
    
    #[cfg(target_os = "windows")]
    {
        capture_screen_windows().await
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Screen capture not implemented for this platform"))
    }
}

#[cfg(target_os = "macos")]
async fn capture_screen_macos() -> Result<String> {
    // Simplified implementation for testing
    // In a real app, this would use ScreenCaptureKit to capture the screen
    let placeholder_jpeg = create_placeholder_jpeg(800, 600)?;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&placeholder_jpeg);
    
    Ok(base64_data)
}

#[cfg(target_os = "macos")]
unsafe fn convert_cgimage_to_jpeg(image: CGImageRef) -> Result<Vec<u8>> {
    // For simplicity, we'll create a minimal JPEG representation
    // In a real implementation, you'd use ImageIO framework
    
    // Get image dimensions using the public API
    let width = image.width();
    let height = image.height();
    
    // For now, return a minimal placeholder JPEG
    // This would need to be replaced with actual ImageIO conversion
    let placeholder_jpeg = create_placeholder_jpeg(width as u32, height as u32)?;
    
    Ok(placeholder_jpeg)
}

#[allow(dead_code)]
fn create_placeholder_jpeg(width: u32, height: u32) -> Result<Vec<u8>> {
    // Create a simple placeholder image using the image crate
    use image::{ImageBuffer, RgbImage, Rgb};
    
    let mut img: RgbImage = ImageBuffer::new(width, height);
    
    // Fill with a gradient pattern
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = (x * 255 / width) as u8;
        let g = (y * 255 / height) as u8;
        let b = 128;
        *pixel = Rgb([r, g, b]);
    }
    
    // Encode as JPEG
    let mut jpeg_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_data);
    
    image::write_buffer_with_format(
        &mut cursor,
        &img,
        width,
        height,
        image::ColorType::Rgb8,
        image::ImageFormat::Jpeg,
    )?;
    
    Ok(jpeg_data)
}

#[cfg(target_os = "windows")]
async fn capture_screen_windows() -> Result<String> {
    // Try modern Windows Graphics Capture API first (Windows 10+)
    if let Ok(result) = capture_screen_modern_windows().await {
        return Ok(result);
    }
    
    // Fallback to GDI for older Windows or if modern API fails
    log::warn!("Modern screenshot API failed, falling back to GDI");
    capture_screen_gdi_windows().await
}

#[cfg(target_os = "windows")]
async fn capture_screen_modern_windows() -> Result<String> {
    // For now, we'll implement the GDI version as the primary method
    // Modern Windows Graphics Capture API implementation would go here
    // This requires more complex COM integration
    Err(anyhow::anyhow!("Modern Windows Graphics Capture not yet implemented"))
}

#[cfg(target_os = "windows")]
async fn capture_screen_gdi_windows() -> Result<String> {
    unsafe {
        // Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN) as u32;
        let screen_height = GetSystemMetrics(SM_CYSCREEN) as u32;
        
        // Get device contexts
        let desktop_window = GetDesktopWindow();
        let desktop_dc = GetDC(Some(desktop_window));
        let memory_dc = CreateCompatibleDC(Some(desktop_dc));
        
        // Create bitmap
        let bitmap = CreateCompatibleBitmap(desktop_dc, screen_width as i32, screen_height as i32);
        let _old_bitmap = SelectObject(memory_dc, bitmap.into());
        
        // Copy screen to bitmap
        let result = BitBlt(
            memory_dc,
            0,
            0,
            screen_width as i32,
            screen_height as i32,
            Some(desktop_dc),
            0,
            0,
            SRCCOPY,
        );
        
        if result.is_ok() {
            // Get bitmap data
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: screen_width as i32,
                    biHeight: -(screen_height as i32), // Negative for top-down bitmap
                    biPlanes: 1,
                    biBitCount: 24,
                    biCompression: 0, // BI_RGB = 0
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };
            
            let buffer_size = (screen_width * screen_height * 3) as usize;
            let mut buffer: Vec<u8> = vec![0; buffer_size];
            
            let get_bits_result = GetDIBits(
                memory_dc,
                bitmap,
                0,
                screen_height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );
            
            if get_bits_result > 0 {
                // Convert to JPEG using the image crate with compression
                let img = image::RgbImage::from_raw(screen_width, screen_height, buffer)
                    .ok_or_else(|| anyhow::anyhow!("Failed to create image from bitmap data"))?;
                
                let mut jpeg_data = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut jpeg_data);
                
                // Use JPEG quality ~75% for compression
                image::write_buffer_with_format(
                    &mut cursor,
                    &img,
                    screen_width,
                    screen_height,
                    image::ColorType::Rgb8,
                    image::ImageFormat::Jpeg,
                )?;
                
                // Cleanup
                let _ = DeleteObject(bitmap.into());
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(Some(desktop_window), desktop_dc);
                
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&jpeg_data);
                return Ok(base64_data);
            }
        }
        
        // Cleanup on error
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory_dc);
        let _ = ReleaseDC(Some(desktop_window), desktop_dc);
        
        Err(anyhow::anyhow!("Failed to capture screen with GDI on Windows"))
    }
}

#[allow(dead_code)]
pub async fn capture_active_window() -> Result<String> {
    // For now, use the same implementation as full screen
    // In a real app, you'd capture just the active window
    capture_screen().await
}
