use std::net::TcpListener;

use crate::utils::terminal::silent_command;

pub fn is_port_available(host: &str, port: u16) -> bool {
    TcpListener::bind((host, port)).is_ok()
}

pub fn find_pid_by_port(port: u16) -> Result<u32, String> {
    #[cfg(windows)]
    {
        let output = silent_command("netstat")
            .args(["-ano", "-p", "tcp"])
            .output()
            .map_err(|err| format!("Failed to inspect network: {}", err))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let port_suffix = format!(":{}", port);

        for line in stdout.lines() {
            // Windows netstat -ano 列: Proto, Local Address, Foreign Address, State, PID
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let proto = parts[0];
            let local_addr = parts[1];
            let state = parts[3];
            let pid = parts[4];

            if proto.eq_ignore_ascii_case("TCP")
                && local_addr.ends_with(&port_suffix)
                && state.eq_ignore_ascii_case("LISTENING")
            {
                return pid
                    .parse::<u32>()
                    .map_err(|e| format!("Invalid PID in netstat output ({}): {}", pid, e));
            }
        }

        Err(format!("No process is listening on port {}.", port))
    }

    #[cfg(unix)]
    {
        // lsof -nP -iTCP:<port> -sTCP:LISTEN -t 只输出监听该端口的 PID
        let port_arg = format!("-iTCP:{}", port);
        let output = silent_command("lsof")
            .args(["-nP", port_arg.as_str(), "-sTCP:LISTEN", "-t"])
            .output()
            .map_err(|err| format!("Failed to inspect network: {}", err))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find_map(|line| line.trim().parse::<u32>().ok())
            .ok_or_else(|| format!("No process is listening on port {}.", port))
    }
}

// 按 PID 强制结束进程：Windows 用 taskkill /T /F（连带子进程），Unix 用 kill -9。
pub fn kill_process_by_pid(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = silent_command("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(|err| format!("Failed to kill DSH service (PID {}): {}", pid, err))?;
        if !status.success() {
            return Err(format!("taskkill failed to terminate DSH service (PID {}).", pid));
        }
    }

    #[cfg(unix)]
    {
        let status = silent_command("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .map_err(|err| format!("Failed to kill DSH service (PID {}): {}", pid, err))?;
        if !status.success() {
            return Err(format!("kill failed to terminate DSH service (PID {}).", pid));
        }
    }

    Ok(())
}
