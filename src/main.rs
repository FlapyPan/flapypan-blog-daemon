#[cfg(windows)]
extern crate winapi;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

use log::{debug, error, info};
use serde::Deserialize;
use tokio::time;

#[cfg(windows)]
fn get_foreground_app_name<'a>() -> Option<String> {
    use std::os::windows::ffi::OsStringExt;
    use winapi::um::{
        handleapi::CloseHandle,
        processthreadsapi::OpenProcess,
        psapi::GetProcessImageFileNameW,
        winnt::PROCESS_QUERY_LIMITED_INFORMATION,
        winuser::{GetForegroundWindow, GetWindowThreadProcessId},
    };
    let foreground_window_handle = unsafe { GetForegroundWindow() };
    debug!("foreground window handle: {:?}.", &foreground_window_handle);
    let mut process_id: u32 = 0;
    unsafe { GetWindowThreadProcessId(foreground_window_handle, &mut process_id); }
    debug!("foreground process id: {:?}.", &process_id);
    let process_handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    if process_handle.is_null() {
        error!("unable to get foreground process.");
        return None;
    }
    debug!("foreground process handle: {:?}", &process_handle);
    let mut buffer = vec![0_u16; 256];
    let result = unsafe { GetProcessImageFileNameW(process_handle, buffer.as_mut_ptr(), buffer.capacity() as u32) };
    unsafe { CloseHandle(process_handle); }
    if result <= 0 {
        error!("unable to get foreground application path.");
        return None;
    }
    unsafe { buffer.set_len(result as usize); }
    let path = OsString::from_wide(&buffer);
    debug!("foreground application path: {:?}", &path);
    Path::new(&path).file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| String::from(file_name))
}

#[cfg(not(windows))]
fn get_foreground_app() -> Option<OsString> {
    error!("current system is not supported");
    None
}

struct Args {
    server: String,
    token: String,
}

#[derive(Deserialize, Debug)]
struct Record {
    name: String,
    description: String,
}

async fn post_state(args: &Args, app_name: String) {
    info!("post state to server: {}", &args.server);
    let client = reqwest::Client::new();
    match client.post(&args.server)
        .header("Authorization", format!("Bearer {}", &args.token))
        .body(app_name)
        .send()
        .await {
        Ok(_) => {}
        Err(err) => { error!("push state error: {:?}", err) }
    };
}

async fn run_interval_task(args: &Args, state_map: &HashMap<String, String>) {
    let mut interval = time::interval(time::Duration::from_secs(10));
    info!("start scheduled task.");
    loop {
        if let Some(app_name) = get_foreground_app_name() {
            info!("app name: {}.",app_name);
            match state_map.get(&app_name.to_ascii_lowercase()) {
                None => {}
                Some(description) => {
                    info!("state: {}.", description);
                    post_state(&args, description.to_string()).await;
                }
            }
        }
        interval.tick().await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let env_server = std::env::var("DAEMON_SERVER")
        .expect("please provide the environment variable: <DAEMON_SERVER>.");
    let env_token = std::env::var("DAEMON_TOKEN")
        .expect("please provide the environment variable: <DAEMON_TOKEN>.");
    let args = Args {
        server: env_server,
        token: env_token,
    };
    let mut reader = csv::Reader::from_path("config.csv")
        .expect("can not load \"config.csv\"");
    let mut state_map: HashMap<String, String> = HashMap::new();
    for record in reader.deserialize() {
        let record: Record = record?;
        state_map.insert(record.name.to_ascii_lowercase(), record.description);
    }
    debug!("state_map: {:?}", &state_map);
    run_interval_task(&args, &state_map).await;
    Ok(())
}
