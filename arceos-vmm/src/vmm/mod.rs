mod config;
#[cfg(feature = "gdb")]
mod gdbserver;
mod images;
mod timer;
mod vcpus;
mod vm_list;

use std::os::arceos::api::task::{self, AxWaitQueueHandle};

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

#[cfg(feature = "gdb")]
use crate::vmm::gdbserver::GdbServer;
#[cfg(feature = "gdb")]
use alloc::boxed::Box;

use crate::hal::{AxVCpuHalImpl, AxVMHalImpl};
pub use timer::init_percpu as init_timer_percpu;

pub type VM = axvm::AxVM<AxVMHalImpl, AxVCpuHalImpl>;
pub type VMRef = axvm::AxVMRef<AxVMHalImpl, AxVCpuHalImpl>;

pub type VCpuRef = axvm::AxVCpuRef<AxVCpuHalImpl>;

static VMM: AxWaitQueueHandle = AxWaitQueueHandle::new();

static RUNNING_VM_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    // Initialize guest VM according to config file.
    config::init_guest_vms();

    // Setup vcpus, spawn axtask for primary VCpu.
    info!("Setting up vcpus...");
    for vm in vm_list::get_vm_list() {
        vcpus::setup_vm_primary_vcpu(vm.clone());
    }
}

pub fn start() {
    info!("VMM starting, booting VMs...");
    for vm in vm_list::get_vm_list() {
        // let gdb_port = Some(5555);
        // Initialize GDB server if port is configured
        #[cfg(feature = "gdb")]
        if let Some(port) = gdb_port {
            info!("Initializing GDB server on port {}", port);
            if let Ok(gdbserver) = GdbServer::new(port) {
                info!("Started GDB server on port {}", port);
                vm.gdbserver_init(Box::new(gdbserver));
            }
        }
        match vm.boot() {
            Ok(_) => {
                vcpus::notify_primary_vcpu(vm.id());
                RUNNING_VM_COUNT.fetch_add(1, Ordering::Release);
                info!("VM[{}] boot success", vm.id())
            }
            Err(err) => warn!("VM[{}] boot failed, error {:?}", vm.id(), err),
        }
    }

    // Do not exit until all VMs are stopped.
    task::ax_wait_queue_wait_until(&VMM, || RUNNING_VM_COUNT.load(Ordering::Acquire) == 0, None);
}
