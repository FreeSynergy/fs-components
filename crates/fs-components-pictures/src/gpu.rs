//! The software Vulkan driver (lavapipe), forced for every picture so that a machine with a real GPU draws the same
//! pixels as the runner.
//!
//! Pattern: none (one function; environment set once before the first adapter is asked for).

use std::path::PathBuf;
use std::sync::OnceLock;

/// Where lavapipe's ICD file may live (distributions differ).
const DIRS: [&str; 4] = [
    "/usr/share/vulkan/icd.d",
    "/etc/vulkan/icd.d",
    "/usr/local/share/vulkan/icd.d",
    "/usr/lib/x86_64-linux-gnu/vulkan/icd.d",
];

/// The ICD file of lavapipe (`lvp_icd*.json`), if installed.
pub fn icd_file() -> Option<PathBuf> {
    DIRS.iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("lvp_icd") && n.rsplit('.').next().is_some_and(|e| e.eq_ignore_ascii_case("json")))
        })
        .min()
}

/// Make Vulkan see only lavapipe and wgpu use only Vulkan. Call it before the first picture of the process; it
/// does the work once. `Ok(name)` is the adapter that will be used, `Err` says why software drawing is not there.
///
/// # Errors
/// No lavapipe ICD file is installed (`mesa-vulkan-drivers`), or the adapter that answers is not a software one.
pub fn force_software() -> Result<&'static str, String> {
    static DONE: OnceLock<Result<String, String>> = OnceLock::new();
    DONE.get_or_init(|| {
        let icd = icd_file().ok_or("lavapipe is not installed (no lvp_icd*.json; package mesa-vulkan-drivers)")?;
        // SAFETY: runs once, from the first picture of the process, before any adapter exists; the tests of this
        // crate draw only through `Group`, which calls this first, and no other thread reads these variables.
        unsafe {
            std::env::set_var("VK_ICD_FILENAMES", &icd);
            std::env::set_var("VK_DRIVER_FILES", &icd);
            std::env::set_var("WGPU_BACKEND", "vulkan");
        }
        adapter_name()
    })
    .as_ref()
    .map(String::as_str)
    .map_err(Clone::clone)
}

fn adapter_name() -> Result<String, String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapters = instance.enumerate_adapters(wgpu::Backends::VULKAN);
    let info = adapters.first().map(wgpu::Adapter::get_info).ok_or("no Vulkan adapter")?;
    if adapters.len() == 1 && info.device_type == wgpu::DeviceType::Cpu {
        Ok(info.name)
    } else {
        Err(format!("the adapter is not the software one: {} ({:?}), {} adapters", info.name, info.device_type, adapters.len()))
    }
}
