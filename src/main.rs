#![windows_subsystem = "windows"]

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::{
	PostQuitMessage,
	DefWindowProcW,
	DestroyWindow,
	GetMessageW,
	DispatchMessageW,
	RegisterClassW,
	CreateWindowExW,
	MSG,
	WM_DESTROY,
	WM_POWERBROADCAST,
	WNDCLASSW,
	HWND_MESSAGE,
	WINDOW_STYLE,
	DEVICE_NOTIFY_WINDOW_HANDLE,
	PBT_POWERSETTINGCHANGE,
    },
    Win32::System::{
	Power::{
	    POWERBROADCAST_SETTING,
	    RegisterPowerSettingNotification,
	    UnregisterPowerSettingNotification,
	    SetSuspendState,
	},
	LibraryLoader::GetModuleHandleW,
	SystemServices::GUID_BATTERY_PERCENTAGE_REMAINING,
    },
};

// ""
const WINDOW_TITLE: &[u16; 1] = &[0];

// "HiddenWindowClass"
const CLASS_NAME: &[u16; 18] = &[72, 105, 100, 100, 101, 110, 87, 105, 110, 100, 111, 119, 67, 108, 97, 115, 115, 0];


fn main() -> Result<()> {
    let hwnd = unsafe {
	let h_instance = HINSTANCE( GetModuleHandleW(None)?.0 );

	let wc = WNDCLASSW {
	    lpfnWndProc: Some(wnd_proc),
	    hInstance: h_instance,
	    lpszClassName: PCWSTR(CLASS_NAME.as_ptr()),
	    ..Default::default()
	};

	RegisterClassW(&wc);

	CreateWindowExW(
	    Default::default(),
	    PCWSTR(CLASS_NAME.as_ptr()),
	    PCWSTR(WINDOW_TITLE.as_ptr()),
	    WINDOW_STYLE::default(),
	    0, 0,
	    0, 0,
	    Some(HWND_MESSAGE),
	    None,
	    Some( h_instance ),
	    None,
	)?
    };

    let notification_handle = unsafe {
	RegisterPowerSettingNotification(
	    HANDLE(hwnd.0),
	    &GUID_BATTERY_PERCENTAGE_REMAINING as *const GUID,
	    DEVICE_NOTIFY_WINDOW_HANDLE
	)?
    };

    let mut msg = MSG::default();
    unsafe {
	while GetMessageW(&mut msg, None, 0, 0).as_bool() {
	    DispatchMessageW(&msg);
	}
	let _ = UnregisterPowerSettingNotification(notification_handle);
        let _ = DestroyWindow(hwnd);
    }

    Ok(())
}

fn process_power_setting_change(lparam: LPARAM) {
    let data = unsafe { &*(lparam.0 as *const POWERBROADCAST_SETTING) };
    if data.PowerSetting == GUID_BATTERY_PERCENTAGE_REMAINING {
	let percentage = data.Data[0];
	if percentage < 65 {
	    let _ = unsafe { SetSuspendState(true, false, false) };
	}
    }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
	    unsafe { PostQuitMessage(0) };
	    LRESULT(0)
        },
	WM_POWERBROADCAST => {
	    if wparam.0.try_into().unwrap_or(0) == PBT_POWERSETTINGCHANGE {
		process_power_setting_change(lparam);
	    }
	    LRESULT(1)
	},
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}