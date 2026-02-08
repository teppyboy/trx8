use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::ptr;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;

use tracing::{debug, error, trace};
use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::System::Services::*;
use windows::Win32::System::Threading::*;
use windows::core::*;

#[link(name = "ntdll")]
unsafe extern "system" {
    unsafe fn NtImpersonateThread(
        ThreadHandle: HANDLE,
        ThreadToImpersonate: HANDLE,
        SecurityQualityOfService: *const SECURITY_QUALITY_OF_SERVICE,
    ) -> NTSTATUS;
}

// I do love the black magic here :)
static IMPERSONATE_TOKEN: OnceLock<Mutex<isize>> = OnceLock::new();

fn set_privilege(h_token: &HANDLE, privilege_name: &str, enable: bool) -> bool {
    unsafe {
        let mut luid = LUID::default();
        let name = HSTRING::from(privilege_name);

        if LookupPrivilegeValueW(PCWSTR::null(), &name, &mut luid).is_err() {
            trace!("LookupPrivilegeValue failed for {}", privilege_name);
            return false;
        }

        let tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: if enable {
                    SE_PRIVILEGE_ENABLED
                } else {
                    windows::Win32::Security::TOKEN_PRIVILEGES_ATTRIBUTES(0)
                },
            }],
        };
        let _ = AdjustTokenPrivileges(*h_token, false, Some(&tp), 0, None, None);
        let result = GetLastError().0 == 0;
        if !result {
            trace!(
                "AdjustTokenPrivileges failed for {}: {}",
                privilege_name,
                GetLastError().0
            );
        }
        result
    }
}

pub fn enable_privileges() -> bool {
    debug!("Enabling required privileges...");
    unsafe {
        let mut h_token = HANDLE::default();
        match OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut h_token,
        ) {
            Ok(_) => {
                for privilege in &[SE_DEBUG_NAME, SE_IMPERSONATE_NAME] {
                    if !set_privilege(&h_token, privilege.to_string().unwrap().as_str(), true) {
                        error!(
                            "Failed to enable privilege: {}",
                            privilege.to_string().unwrap()
                        );
                        return false;
                    } else {
                        trace!(
                            "Successfully enabled privilege: {}",
                            privilege.to_string().unwrap()
                        );
                    }
                }
                let _ = CloseHandle(h_token);
            }
            Err(_) => {
                error!("Failed to open current process token.");
                return false;
            }
        }
    };
    return true;
}

fn find_process_by_name(name: &str) -> Option<u32> {
    // Using std::process::Command to query system state via `tasklist` avoids the heavy Toolhelp32 API boilerplate.
    match Command::new("cmd")
        .args([
            "/C",
            "tasklist",
            "/FI",
            &format!("IMAGENAME eq {}", name),
            "/FO",
            "CSV",
            "/NH",
        ])
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
    {
        Ok(output) => {
            let content = String::from_utf8_lossy(&output.stdout);
            // Parse CSV output: "winlogon.exe","1234","Services","0","24,000 K"
            if let Some(line) = content.lines().next() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let pid_str = parts[1].trim_matches('"');
                    return pid_str.parse::<u32>().ok();
                }
            }
            None
        }
        Err(e) => {
            // Fallback to Win32 if tasklist fails (unlikely, but safe)
            error!("Find process by tasklist failed: {}", e);
            None
        }
    }
}

pub fn impersonate_system() -> bool {
    debug!("Attempting to impersonate SYSTEM...");
    unsafe {
        if let Some(pid_winlogon) = find_process_by_name("winlogon.exe") {
            trace!("Found winlogon PID: {}", pid_winlogon);
            let h_winlogon = match OpenProcess(
                PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION,
                false,
                pid_winlogon,
            ) {
                Ok(handle) => handle,
                Err(e) => {
                    error!("Failed to open winlogon process: {}", e);
                    return false;
                }
            };
            let mut h_sys_tkn = HANDLE::default();
            if OpenProcessToken(h_winlogon, TOKEN_QUERY | TOKEN_DUPLICATE, &mut h_sys_tkn).is_ok() {
                let success;
                match ImpersonateLoggedOnUser(h_sys_tkn) {
                    Ok(_) => {
                        trace!("Successfully impersonated SYSTEM.");
                        success = true;
                    }
                    Err(e) => {
                        error!("Failed to impersonate SYSTEM: {}", e);
                        success = false;
                    }
                }
                let _ = CloseHandle(h_sys_tkn);
                let _ = CloseHandle(h_winlogon);
                return success;
            }
            let _ = CloseHandle(h_winlogon);
        }
        false
    }
}

///
/// Code generated by Z.ai GLM-4.7, based on the original VBScript code below:
///
/// https://github.com/fafalone/RunAsTrustedInstaller
///
pub fn impersonate_ti() -> bool {
    debug!("Attempting to impersonate TrustedInstaller...");
    unsafe {
        let sc_manager = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS)
        {
            Ok(handle) => handle,
            Err(e) => {
                error!("Failed to open SCManager: {}", e);
                return false;
            }
        };

        let service_name = HSTRING::from("TrustedInstaller");
        let service = match OpenServiceW(
            sc_manager,
            &service_name,
            SERVICE_START | SERVICE_QUERY_STATUS,
        ) {
            Ok(handle) => handle,
            Err(e) => {
                error!("Failed to open TrustedInstaller service: {}", e);
                let _ = CloseServiceHandle(sc_manager);
                return false;
            }
        };

        let mut bytes_needed = 0u32;
        let mut status_proc = SERVICE_STATUS_PROCESS::default();
        let mut ti_pid: u32 = 0;

        // Poll service status
        loop {
            let status_bytes = std::slice::from_raw_parts_mut(
                &mut status_proc as *mut _ as *mut u8,
                mem::size_of::<SERVICE_STATUS_PROCESS>(),
            );
            if QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                Some(status_bytes),
                &mut bytes_needed,
            )
            .is_ok()
            {
                match status_proc.dwCurrentState {
                    SERVICE_STOPPED => {
                        trace!("Starting TrustedInstaller...");
                        let _ = StartServiceW(service, None);
                        std::thread::sleep(Duration::from_millis(500));
                    }
                    SERVICE_RUNNING => {
                        ti_pid = status_proc.dwProcessId;
                        trace!("TrustedInstaller running, PID: {}", ti_pid);
                        break;
                    }
                    _ => {
                        std::thread::sleep(Duration::from_millis(status_proc.dwWaitHint as u64));
                    }
                }
            } else {
                break;
            }
        }

        let mut h_ti_token: HANDLE = HANDLE::default();

        if ti_pid > 0 {
            if let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) {
                let mut entry = THREADENTRY32 {
                    dwSize: mem::size_of::<THREADENTRY32>() as u32,
                    ..Default::default()
                };

                if Thread32First(snapshot, &mut entry).is_ok() {
                    loop {
                        if entry.th32OwnerProcessID == ti_pid {
                            if let Ok(h_thread) =
                                OpenThread(THREAD_DIRECT_IMPERSONATION, false, entry.th32ThreadID)
                            {
                                let sqos = SECURITY_QUALITY_OF_SERVICE {
                                    Length: mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32,
                                    ImpersonationLevel: SecurityImpersonation,
                                    ContextTrackingMode: 0,
                                    EffectiveOnly: false.into(),
                                };
                                if NtImpersonateThread(GetCurrentThread(), h_thread, &sqos)
                                    == STATUS_SUCCESS
                                {
                                    let _ = OpenThreadToken(
                                        GetCurrentThread(),
                                        TOKEN_ALL_ACCESS,
                                        false,
                                        &mut h_ti_token,
                                    );
                                }
                                let _ = CloseHandle(h_thread);
                                break;
                            }
                        }
                        if Thread32Next(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }
                let _ = CloseHandle(snapshot);
            }
        }

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(sc_manager);
        if !h_ti_token.is_invalid() {
            IMPERSONATE_TOKEN.set(Mutex::new(h_ti_token.0 as isize)).ok();
            trace!("Successfully impersonated TrustedInstaller, TI token: 0x{:x}", h_ti_token.0 as usize);
            return true;
        } else {
            error!("Failed to impersonate TrustedInstaller.");
            return false;
        }
    }
}

pub fn launch_as_ti(command: String, args: String, cwd: Option<String>) -> bool {
    unsafe {
        if IMPERSONATE_TOKEN.get().is_none() {
            impersonate_ti();
        }

        let h_ti_token = match IMPERSONATE_TOKEN.get() {
            Some(ref t) => {
                let token_value = *t.lock().unwrap();
                if token_value != 0 {
                    HANDLE(token_value as *mut _)
                } else {
                    error!("No valid impersonation token available.");
                    return false;
                }
            },
            _ => {
                error!("Failed to acquire token.");
                return false;
            }
        };

        let mut h_stolen_token = HANDLE::default();
        let security_attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: BOOL::from(false),
        };

        if DuplicateTokenEx(
            h_ti_token,
            TOKEN_ALL_ACCESS,
            Some(&security_attributes),
            SecurityImpersonation,
            TokenImpersonation,
            &mut h_stolen_token,
        ).is_ok() {
            
            let startup_info = STARTUPINFOW {
                cb: mem::size_of::<STARTUPINFOW>() as u32,
                lpDesktop: PWSTR(w!("WinSta0\\Default").0 as *mut _),
                ..Default::default()
            };

            let mut process_info = PROCESS_INFORMATION::default();
            let mut command_wide: Vec<u16> = OsString::from(&command).encode_wide().chain([0]).collect();
            let mut args_wide: Vec<u16> = OsString::from(&args).encode_wide().chain([0]).collect();
            let mut cwd_wide: Vec<u16> = if let Some(ref cwd_str) = cwd {
                OsString::from(cwd_str).encode_wide().chain([0]).collect()
            } else {
                Vec::new()
            };
            let flags = CREATE_UNICODE_ENVIRONMENT;

            let result = CreateProcessWithTokenW(
                h_stolen_token,
                LOGON_WITH_PROFILE,
                PWSTR(command_wide.as_mut_ptr()),
                Some(PWSTR(args_wide.as_mut_ptr())),
                flags,
                None,
                // Risky af, and not tested yet :)
                PWSTR(cwd_wide.as_mut_ptr()),
                &startup_info,
                &mut process_info
            );

            let success = result.is_ok();
            if !success {
                let err = GetLastError();
                error!("CreateProcessWithTokenW failed: {:?}", err);
            }

            let _ = CloseHandle(h_stolen_token);
            
            if success {
                // Wait for the process to complete
                trace!("Waiting for process to complete...");
                let wait_result = WaitForSingleObject(process_info.hProcess, INFINITE);
                
                if wait_result == WAIT_OBJECT_0 {
                    let mut exit_code: u32 = 0;
                    if GetExitCodeProcess(process_info.hProcess, &mut exit_code).is_ok() {
                        trace!("Process exited with code: {}", exit_code);
                    }
                } else {
                    error!("WaitForSingleObject failed or timed out: {:?}", wait_result);
                }
                
                let _ = CloseHandle(process_info.hProcess);
                let _ = CloseHandle(process_info.hThread);
            } else {
                error!("Failed to launch process '{}' as TrustedInstaller.", command);
            }

            success
        } else {
            error!("DuplicateTokenEx failed.");
            false
        }
    }
}
