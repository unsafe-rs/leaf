use std::process::Command;

use log::debug;
use sysproxy::Sysproxy;

pub fn osascript() {
    Command::new("osascript")
        .arg("-e")
        .arg("do shell script 'helloword' with administrator privileges")
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}

pub fn set_system_proxy(http_port: u16, socks_port: u16) -> anyhow::Result<()> {
    debug!("set_system_proxy");
    // #[cfg(target_os = "macos")]
    // {
    //     Command::new("osascript")
    //         .arg("-e")
    //         .arg(format!(
    //             "do shell script \"{} {} && {} && {} {} && {}\" with administrator privileges",
    //             "networksetup -setwebproxy wi-fi 127.0.0.1",
    //             http_port,
    //             "networksetup -setwebproxystate wi-fi on",
    //             "networksetup -setsocksfirewallproxy wi-fi 127.0.0.1",
    //             socks_port,
    //             "networksetup -setsocksfirewallproxystate wi-fi on",
    //         ))
    //         .spawn()?
    //         .wait_with_output()?;
    // }

    Sysproxy {
        enable: true,
        host: "127.0.0.1".into(),
        socks_port: Some(socks_port),
        ..Default::default()
    }
    .set_system_proxy()?;

    Ok(())
}

pub fn unset_system_proxy() -> anyhow::Result<()> {
    debug!("unset_system_proxy");
    // #[cfg(target_os = "macos")]
    // {
    //     Command::new("osascript")
    //         .arg("-e")
    //         .arg(format!(
    //             "do shell script \"{} && {}\" with administrator privileges",
    //             "networksetup -setwebproxystate wi-fi off",
    //             "networksetup -setsocksfirewallproxystate wi-fi off",
    //         ))
    //         .spawn()?
    //         .wait_with_output()?;
    // }

    Sysproxy {
        enable: false,
        host: "".into(),
        http_port: Some(0),
        https_port: Some(0),
        socks_port: Some(0),
        ..Default::default()
    }
    .set_system_proxy()?;

    Ok(())
}
