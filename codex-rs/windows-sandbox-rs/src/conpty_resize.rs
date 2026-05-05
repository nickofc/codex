use anyhow::Result;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Console::COORD;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
use windows_sys::Win32::System::LibraryLoader::LoadLibraryA;

type ResizePseudoConsoleFn = unsafe extern "system" fn(HANDLE, COORD) -> i32;

fn resize_pseudoconsole_fn() -> Option<ResizePseudoConsoleFn> {
    static RESIZE_PSEUDOCONSOLE: OnceLock<Option<ResizePseudoConsoleFn>> = OnceLock::new();
    *RESIZE_PSEUDOCONSOLE.get_or_init(|| {
        let kernel32 = unsafe {
            let loaded = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
            if loaded == 0 {
                LoadLibraryA(b"kernel32.dll\0".as_ptr())
            } else {
                loaded
            }
        };
        if kernel32 == 0 {
            return None;
        }

        let proc = unsafe {
            GetProcAddress(
                kernel32,
                b"ResizePseudoConsole\0".as_ptr(),
            )
        }?;
        Some(unsafe { std::mem::transmute::<_, ResizePseudoConsoleFn>(proc) })
    })
}

pub fn resize_pseudoconsole(hpc: HANDLE, rows: u16, cols: u16) -> Result<()> {
    let resize = resize_pseudoconsole_fn().ok_or_else(|| {
        anyhow::anyhow!("ResizePseudoConsole is not available on this Windows version")
    })?;
    let result = unsafe {
        resize(
            hpc,
            COORD {
                X: cols as i16,
                Y: rows as i16,
            },
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "failed to resize console: HRESULT {result}"
        ))
    }
}
