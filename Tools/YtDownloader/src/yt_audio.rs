use anyhow::{Result, Context, bail};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};
use std::io::Write;

#[cfg(not(target_os = "linux"))]
use std::process::Stdio;
#[cfg(not(target_os = "linux"))]
use tokio::process::Command;
#[cfg(not(target_os = "linux"))]
use tokio::io::{AsyncBufReadExt, BufReader};
#[cfg(not(target_os = "linux"))]
use uuid::Uuid;

#[cfg(target_os = "linux")]
use std::io::{Seek};
#[cfg(target_os = "linux")]
use anyhow::anyhow;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
static YOUTUBE_EXPLODE_BIN: &[u8] = include_bytes!("../assets/win-x64/YtExplodeCli.exe");
#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
static YOUTUBE_EXPLODE_BIN: &[u8] = include_bytes!("../assets/win-arm64/YtExplodeCli.exe");
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
static YOUTUBE_EXPLODE_BIN: &[u8] = include_bytes!("../assets/linux-x64/YtExplodeCli");
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
static YOUTUBE_EXPLODE_BIN: &[u8] = include_bytes!("../assets/linux-arm64/YtExplodeCli");
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
static YOUTUBE_EXPLODE_BIN: &[u8] = include_bytes!("../assets/osx-x64/YtExplodeCli");
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
static YOUTUBE_EXPLODE_BIN: &[u8] = include_bytes!("../assets/osx-arm64/YtExplodeCli");

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub file_path: Option<PathBuf>,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct YoutubeExplode {
    #[cfg(not(target_os = "linux"))]
    exe_path: Arc<PathBuf>,
    #[cfg(not(target_os = "linux"))]
    _temp_guard: Arc<TempCleanup>,
    logging_enabled: Arc<Mutex<bool>>,
}

#[cfg(not(target_os = "linux"))]
struct TempCleanup {
    path: PathBuf,
}

#[cfg(not(target_os = "linux"))]
impl Drop for TempCleanup {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl YoutubeExplode {
    pub fn new(logging_enabled: bool) -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            Ok(Self {
                logging_enabled: Arc::new(Mutex::new(logging_enabled)),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
            let temp_path = std::env::temp_dir().join(format!(
                "youtube-explode-{}{}",
                Uuid::new_v4(),
                exe_suffix
            ));

            {
                let mut file = std::fs::File::create(&temp_path)
                    .context("Failed to create temp file for YoutubeExplode")?;
                file.write_all(YOUTUBE_EXPLODE_BIN)
                    .context("Failed to write embedded YoutubeExplode binary")?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perm = std::fs::Permissions::from_mode(0o755);
                std::fs::set_permissions(&temp_path, perm)
                    .context("Failed to set execute permissions on YoutubeExplode binary")?;
            }

            Ok(Self {
                exe_path: Arc::new(temp_path.clone()),
                _temp_guard: Arc::new(TempCleanup { path: temp_path }),
                logging_enabled: Arc::new(Mutex::new(logging_enabled)),
            })
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        #[cfg(not(target_os = "linux"))]
        {
            Some((*self.exe_path).clone())
        }
        #[cfg(target_os = "linux")]
        {
            None
        }
    }

    pub fn set_logging(&self, enabled: bool) {
        let mut flag = self.logging_enabled.lock().unwrap();
        *flag = enabled;
    }

    fn log(&self, msg: &str) {
        if *self.logging_enabled.lock().unwrap() {
            println!("{}", msg);
        }
    }

    // ===== LINUX: run embedded binary entirely from memory =====
    #[cfg(target_os = "linux")]
    async fn run_in_memory(&self, args: Vec<String>) -> Result<(String, String)> {
        use memfd::MemfdOptions;
        use nix::unistd::{fork, ForkResult, pipe, close};
        use nix::sys::wait::{waitpid, WaitStatus};
        use tokio::task;
        use std::ffi::CString;
        use std::fs::File;
        use std::io::Read;
        use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

        let bytes = YOUTUBE_EXPLODE_BIN.to_vec();

        task::spawn_blocking(move || -> Result<(String, String)> {
            let m = MemfdOptions::default()
                .close_on_exec(true)
                .create("yt_explode_mem")
                .context("memfd_create failed")?;
            let mut mem = m.into_file();
            mem.write_all(&bytes).context("writing to memfd failed")?;
            mem.flush().context("flush failed")?;
            mem.seek(std::io::SeekFrom::Start(0)).context("seek failed")?;

            let fd = mem.as_raw_fd();
            let rc = unsafe { libc::fchmod(fd, 0o755) };
            if rc != 0 {
                return Err(anyhow!("fchmod failed: {}", std::io::Error::last_os_error()));
            }

            // Consume OwnedFd into raw i32s immediately
            // Create pipes and convert to raw fds immediately
            let (out_r_owned, out_w_owned) = pipe().context("pipe stdout failed")?;
            let (err_r_owned, err_w_owned) = pipe().context("pipe stderr failed")?;
            
            // Convert OwnedFds to raw fds - this transfers ownership
            let out_r_fd = out_r_owned.into_raw_fd();
            let out_w_fd = out_w_owned.into_raw_fd();
            let err_r_fd = err_r_owned.into_raw_fd();
            let err_w_fd = err_w_owned.into_raw_fd();

            match unsafe { fork().context("fork failed")? } {
                ForkResult::Parent { child } => {
                    // Parent: close write ends (we don’t write)
                    let _ = close(out_w_fd);
                    let _ = close(err_w_fd);

                    // Wrap read ends in File (File now owns those fds)
                    let mut stdout_file = unsafe { File::from_raw_fd(out_r_fd) };
                    let mut stderr_file = unsafe { File::from_raw_fd(err_r_fd) };

                    let stdout_handle = std::thread::spawn(move || {
                        let mut s = String::new();
                        let _ = stdout_file.read_to_string(&mut s);
                        s
                    });

                    let stderr_handle = std::thread::spawn(move || {
                        let mut s = String::new();
                        let _ = stderr_file.read_to_string(&mut s);
                        s
                    });

                    let wait_status = waitpid(child, None).context("waitpid failed")?;
                    let stdout = stdout_handle.join().unwrap_or_default();
                    let stderr = stderr_handle.join().unwrap_or_default();

                    match wait_status {
                        WaitStatus::Exited(_, code) => {
                            if code == 0 {
                                Ok((stdout, stderr))
                            } else {
                                Err(anyhow!("child exited with code {}. stderr: {}", code, stderr))
                            }
                        }
                        WaitStatus::Signaled(_, sig, _) => {
                            Err(anyhow!("child killed by signal {:?}. stderr: {}", sig, stderr))
                        }
                        _ => Err(anyhow!(
                        "unexpected wait status: {:?}. stderr: {}",
                        wait_status,
                        stderr
                    )),
                    }
                }

                ForkResult::Child => {
                    // Child: close the read ends
                    let _ = close(out_r_fd);
                    let _ = close(err_r_fd);

                    // dup2 to stdout/stderr
                    if unsafe { libc::dup2(out_w_fd, libc::STDOUT_FILENO) } == -1 {
                        eprintln!("dup2 stdout failed: {}", std::io::Error::last_os_error());
                        std::process::exit(111);
                    }
                    if unsafe { libc::dup2(err_w_fd, libc::STDERR_FILENO) } == -1 {
                        eprintln!("dup2 stderr failed: {}", std::io::Error::last_os_error());
                        std::process::exit(111);
                    }

                    // Close originals after dup2
                    let _ = close(out_w_fd);
                    let _ = close(err_w_fd);

                    // Build argv/envp
                    let cstr_args: Vec<CString> =
                        args.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
                    let mut argv: Vec<*const libc::c_char> =
                        cstr_args.iter().map(|s| s.as_ptr()).collect();
                    argv.push(std::ptr::null());

                    let envcstrings: Vec<CString> = std::env::vars()
                        .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
                        .collect();
                    let mut envp: Vec<*const libc::c_char> =
                        envcstrings.iter().map(|s| s.as_ptr()).collect();
                    envp.push(std::ptr::null());

                    unsafe {
                        libc::fexecve(fd, argv.as_ptr(), envp.as_ptr());
                    }

                    eprintln!("fexecve failed: {}", std::io::Error::last_os_error());
                    std::process::exit(111);
                }
            }
        })
            .await?
    }


    // ===== check_installed =====
    #[cfg(target_os = "linux")]
    pub async fn check_installed(&self) -> Result<()> {
        let args = vec!["yt-explode".to_string(), "--version".to_string()];
        let (stdout, stderr) = self.run_in_memory(args).await?;
        if !stderr.trim().is_empty() {
            bail!("YoutubeExplode CLI failed: {}", stderr);
        }
        if stdout.trim().is_empty() {
            bail!("YoutubeExplode CLI produced no output for --version");
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn check_installed(&self) -> Result<()> {
        let output = Command::new(&*self.exe_path)
            .arg("--version")
            .output()
            .await
            .context("Failed to execute YoutubeExplode CLI")?;

        if !output.status.success() {
            bail!("YoutubeExplode CLI did not respond successfully");
        }

        Ok(())
    }

    // ===== search =====
    #[cfg(target_os = "linux")]
    pub async fn search(&self, query: &str, limit: usize) -> Result<String> {
        let args = vec![
            "yt-explode".to_string(),
            "search".to_string(),
            query.to_string(),
            "--limit".to_string(),
            limit.to_string(),
        ];
        let (stdout, stderr) = self.run_in_memory(args).await?;
        if !stderr.trim().is_empty() {
            bail!("Search failed: {}", stderr);
        }
        Ok(stdout)
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn search(&self, query: &str, limit: usize) -> Result<String> {
        let output = Command::new(&*self.exe_path)
            .args(["search", query, "--limit", &limit.to_string()])
            .output()
            .await
            .context("Failed to run YoutubeExplode search command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Search failed: {}", error);
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    // ===== download_audio =====
    pub async fn download_audio(&self, url: &str, output_dir: Option<&PathBuf>) -> Result<DownloadResult> {
        sleep(Duration::from_millis(500)).await;

        let out_dir = output_dir.cloned().unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir)
                .context(format!("Failed to create output directory: {}", out_dir.display()))?;
        }

        #[cfg(target_os = "linux")]
        {
            let args = vec![
                "yt-explode".to_string(),
                "download".to_string(),
                url.to_string(),
                "--audio-only".to_string(),
                "--output".to_string(),
                out_dir.to_string_lossy().to_string(),
            ];

            let (stdout, stderr) = self.run_in_memory(args).await?;
            for line in stdout.lines() {
                self.log(line);
            }
            for line in stderr.lines() {
                self.log(line);
            }

            let err_text = stderr.trim().to_string();
            if !err_text.is_empty() {
                if err_text.contains("HTTP") || err_text.contains("403") || err_text.contains("404") || err_text.contains("429") {
                    return Ok(DownloadResult { success: false, file_path: None, error_message: Some(format!("Download failed: {}", err_text)) });
                }
            }

            let latest_file = std::fs::read_dir(&out_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|e| e.path().is_file())
                .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

            Ok(match latest_file {
                Some(file) => DownloadResult { success: true, file_path: Some(file.path()), error_message: None },
                None => DownloadResult { success: false, file_path: None, error_message: Some("Download finished, but no audio file found in output directory".to_string()) },
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let mut cmd = Command::new(&*self.exe_path);
            cmd.args(["download", url, "--audio-only", "--output"])
                .arg(&out_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd.spawn().context("Failed to spawn YoutubeExplode download process")?;
            let stdout = child.stdout.take().map(BufReader::new);
            let stderr = child.stderr.take().map(BufReader::new);

            let mut out_buf = String::new();
            let mut err_buf = String::new();

            if let Some(reader) = stdout {
                let mut lines = reader.lines();
                while let Some(line) = lines.next_line().await? {
                    self.log(&line);
                    out_buf.push_str(&line);
                    out_buf.push('\n');
                }
            }

            if let Some(reader) = stderr {
                let mut lines = reader.lines();
                while let Some(line) = lines.next_line().await? {
                    self.log(&line);
                    err_buf.push_str(&line);
                    err_buf.push('\n');
                }
            }

            let status = child.wait().await.context("Failed to wait for download process")?;

            if !status.success() {
                let err_text = err_buf.trim().to_string();
                let message = if err_text.contains("HTTP") || err_text.contains("403") || err_text.contains("404") || err_text.contains("429") {
                    format!("Download failed: {}", err_text)
                } else {
                    format!("Audio download failed with exit code {:?}\n\nStdout:\n{}\n\nStderr:\n{}", status.code(), out_buf.trim(), err_text)
                };

                return Ok(DownloadResult { success: false, file_path: None, error_message: Some(message) });
            }

            let latest_file = std::fs::read_dir(&out_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|e| e.path().is_file())
                .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

            Ok(match latest_file {
                Some(file) => DownloadResult { success: true, file_path: Some(file.path()), error_message: None },
                None => DownloadResult { success: false, file_path: None, error_message: Some("Download finished, but no audio file found in output directory".to_string()) },
            })
        }
    }
}
