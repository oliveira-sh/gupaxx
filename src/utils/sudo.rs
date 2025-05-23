// Gupaxx - Fork of Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Handling of [sudo] for XMRig.
// [zeroize] is used to wipe the memory after use.
// Only gets imported in [main.rs] for Unix.

use crate::{
    constants::*,
    disk::state::{P2pool, Xmrig, XmrigProxy},
    helper::{Helper, ProcessSignal},
};
use enclose::enc;
use log::*;
use std::{
    io::Write,
    path::Path,
    process::*,
    sync::{Arc, Mutex},
    thread,
};
use zeroize::Zeroize;

#[allow(dead_code)] // for dummy value windows
#[derive(Debug, Clone)]
pub struct SudoState {
    pub windows: bool, // If this bool is set, this struct is just a dummy so I don't have to change my type signatures :)
    pub testing: bool, // Are we attempting a sudo test right now?
    pub success: bool, // Was the sudo test a success?
    pub hide: bool,    // Are we hiding the password?
    pub msg: String,   // The message shown to the user if unsuccessful
    pub pass: String,  // The actual password wrapped in a [SecretVec]
    pub signal: ProcessSignal, // Main GUI will set this depending on if we want [Start] or [Restart]
}

impl Default for SudoState {
    fn default() -> Self {
        Self::new()
    }
}

impl SudoState {
    #[cold]
    #[inline(never)]
    #[cfg(target_os = "windows")]
    pub fn new() -> Self {
        Self {
            windows: true,
            testing: false,
            success: false,
            hide: true,
            msg: String::new(),
            pass: String::new(),
            signal: ProcessSignal::None,
        }
    }
    #[cold]
    #[inline(never)]
    #[cfg(target_family = "unix")]
    pub fn new() -> Self {
        Self {
            windows: false,
            testing: false,
            success: false,
            hide: true,
            msg: "".to_string(),
            pass: String::with_capacity(256),
            signal: ProcessSignal::None,
        }
    }

    #[cold]
    #[inline(never)]
    // Resets the state.
    pub fn reset(state: &Arc<Mutex<Self>>) {
        Self::wipe(state);
        let mut state = state.lock().unwrap();
        state.testing = false;
        state.success = false;
        //		state.signal = ProcessSignal::None;
    }

    #[cold]
    #[inline(never)]
    // Swaps the pass with another 256-capacity String,
    // zeroizes the old and drops it.
    pub fn wipe(state: &Arc<Mutex<Self>>) {
        let mut new = String::with_capacity(256);
        // new is now == old, and vice-versa.
        std::mem::swap(&mut new, &mut state.lock().unwrap().pass);
        // we're wiping & dropping the old pass here.
        new.zeroize();
        std::mem::drop(new);
        info!("Sudo | Password wipe with 0's ... OK");
    }

    #[cold]
    #[inline(never)]
    // Spawns a thread and tests sudo with the provided password.
    // Sudo takes the password through STDIN via [--stdin].
    // Sets the appropriate state fields on success/failure.
    pub fn test_sudo(
        state: Arc<Mutex<Self>>,
        helper: &Arc<Mutex<Helper>>,
        xmrig: &Xmrig,
        p2pool: &P2pool,
        proxy: &XmrigProxy,
        path: &Path,
    ) {
        let path = path.to_path_buf();
        thread::spawn(enc!((helper, xmrig, p2pool, proxy)  move || {
            // Set to testing
            state.lock().unwrap().testing = true;

            // Make sure sudo timestamp is reset
            let reset = Command::new("sudo")
                .arg("--reset-timestamp")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .status();
            match reset {
                Ok(_) => info!("Sudo | Resetting timestamp ... OK"),
                Err(e) => {
                    error!("Sudo | Couldn't reset timestamp: {}", e);
                    Self::wipe(&state);
                    state.lock().unwrap().msg = format!("Sudo error: {}", e);
                    state.lock().unwrap().testing = false;
                    return;
                }
            }

            // Spawn testing sudo
            let mut sudo = Command::new("sudo")
                .args(["--stdin", "--validate"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();

            // Write pass to STDIN
            let mut stdin = sudo.stdin.take().unwrap();
            stdin
                .write_all(state.lock().unwrap().pass.as_bytes())
                .unwrap();
            drop(stdin);

            // Sudo re-prompts and will hang.
            // To workaround this, try checking
            // results for 5 seconds in a loop.
            for i in 1..=5 {
                match sudo.try_wait() {
                    Ok(Some(code)) => {
                        if code.success() {
                            info!("Sudo | Password ... OK!");
                            state.lock().unwrap().success = true;
                            break;
                        }
                    }
                    Ok(None) => {
                        info!("Sudo | Waiting [{}/5]...", i);
                        std::thread::sleep(SECOND);
                    }
                    Err(e) => {
                        error!("Sudo | Couldn't reset timestamp: {}", e);
                        Self::wipe(&state);
                        state.lock().unwrap().msg = format!("Sudo error: {}", e);
                        state.lock().unwrap().testing = false;
                        return;
                    }
                }
            }
            if let Err(e) = sudo.kill() {
                warn!("Sudo | Kill error (it probably already exited): {}", e);
            }
            if state.lock().unwrap().success {
                match state.lock().unwrap().signal {
                    ProcessSignal::Restart => crate::helper::Helper::restart_xmrig(
                        &helper,
                        &xmrig,
                        &p2pool,
                        &proxy,
                        &path,
                        Arc::clone(&state),
                    ),
                    ProcessSignal::Stop => crate::helper::Helper::stop_xmrig(&helper),
                    _ => crate::helper::Helper::start_xmrig(
                        &helper,
                        &xmrig,
                        &p2pool,
                        &proxy,
                        &path,
                        Arc::clone(&state),
                    ),
                }
            } else {
                state.lock().unwrap().msg = "Incorrect password! (or sudo timeout)".to_string();
                Self::wipe(&state);
            }
            state.lock().unwrap().signal = ProcessSignal::None;
            state.lock().unwrap().testing = false;
        }));
    }
}
