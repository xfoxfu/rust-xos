use alloc::boxed::Box;
use core::any::{Any, TypeId};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use hashbrown::HashMap;
use spin::{Mutex, MutexGuard, RwLock};

lazy_static! {
    pub static ref DRIVERS: RwLock<DriverHolder> = RwLock::new(DriverHolder::new());
}

pub struct DriverHolder {
    inner: HashMap<TypeId, Mutex<Box<dyn Any + Send>>>,
}

#[derive(Debug)]
pub enum HolderError {
    DriverAlreadyExist,
    DriverNotExist,
    FailedToLock,
}

const DEFAULT_CAPACITY: usize = 64;

impl DriverHolder {
    pub fn new() -> Self {
        Self {
            inner: HashMap::with_capacity(DEFAULT_CAPACITY),
        }
    }

    pub fn add<T: 'static + Send>(&mut self, value: T) -> Result<(), HolderError> {
        self.inner
            .insert(TypeId::of::<T>(), Mutex::new(Box::from(value)));

        Ok(())
    }

    pub fn get<T: 'static + Send>(&self) -> Result<Guarded<'_, T>, HolderError> {
        let mutex = self
            .inner
            .get(&TypeId::of::<T>())
            .ok_or(HolderError::DriverNotExist)?;
        let mg = mutex.try_lock().ok_or(HolderError::FailedToLock)?;
        Ok(Guarded::new(mg))
    }
}

pub struct Guarded<'a, T: 'static>(MutexGuard<'a, Box<dyn Any + Send>>, PhantomData<T>);

impl<'a, T: 'static> Guarded<'a, T> {
    fn new(mg: MutexGuard<'a, Box<dyn Any + Send>>) -> Self {
        Self(mg, PhantomData)
    }
}

impl<'a, T: 'static> Deref for Guarded<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref().downcast_ref().unwrap()
    }
}

impl<'a, T: 'static> DerefMut for Guarded<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut().downcast_mut().unwrap()
    }
}
