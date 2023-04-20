use crate::{println, synchronization::Spinlock};

const NUM_DRIVERS: usize = 5;

struct DriverManngerInner {
    next_index: usize,
    descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
}

pub mod interface {
    pub trait DeviceDriver {
        fn compatible(&self) -> &'static str;

        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }
}

pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

#[derive(Copy, Clone)]
pub struct DeviceDriverDescriptor {
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
}

impl DeviceDriverDescriptor {
    pub fn new(
        device_driver: &'static (dyn interface::DeviceDriver + Sync),
        post_init_callback: Option<DeviceDriverPostInitCallback>,
    ) -> Self {
        Self {
            device_driver,
            post_init_callback,
        }
    }
}

pub struct DriverManager {
    inner: Spinlock<DriverManngerInner>,
}

static DRIVER_MANAGER: DriverManager = DriverManager::new();

pub fn driver_manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}

impl DriverManngerInner {
    pub const fn new() -> Self {
        Self {
            next_index: 0,
            descriptors: [None; NUM_DRIVERS],
        }
    }
}

impl DriverManager {
    pub const fn new() -> Self {
        Self {
            inner: Spinlock::new(DriverManngerInner::new()),
        }
    }

    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
        let mut locked = self.inner.lock();
        let idx = locked.next_index;
        locked.descriptors[idx] = Some(descriptor);
        locked.next_index += 1;
    }

    fn for_each_descriptor(&self, f: impl FnMut(&DeviceDriverDescriptor)) {
        let locked = self.inner.lock();
        locked
            .descriptors
            .iter()
            .filter_map(|x| x.as_ref())
            .for_each(f)
    }

    pub unsafe fn init_drivers(&self) {
        self.for_each_descriptor(|descriptor| {
            if let Err(e) = descriptor.device_driver.init() {
                panic!(
                    "Error initializing driver {}:{}",
                    descriptor.device_driver.compatible(),
                    e
                );
            }

            if let Some(cb) = &descriptor.post_init_callback {
                if let Err(e) = cb() {
                    panic!(
                        "Error during driver post-init callback {}:{}",
                        descriptor.device_driver.compatible(),
                        e
                    );
                }
            }
        });
    }

    pub fn enumerate(&self) {
        let mut i: usize = 1;
        self.for_each_descriptor(|descriptor| {
            println!("{}:{}", i, descriptor.device_driver.compatible());
            i += 1;
        });
    }
}
