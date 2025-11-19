use std::sync::OnceLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::repository::Repositories;

pub trait Service: Any + Send + Sync {
  fn new_service(services: &Services, repositories: &Repositories) -> Self;
}

pub struct Services {
  services: Mutex<HashMap<TypeId, Arc<OnceLock<Arc<dyn Any + Send + Sync>>>>>,
  repositories: Arc<Repositories>,
}

impl Services {
  pub fn new(repositories: Arc<Repositories>) -> Self {
    Self {
      services: Mutex::new(HashMap::new()),
      repositories,
    }
  }

  pub fn get_service<T>(&self) -> Arc<T>
  where
    T: Service + Send + Sync + 'static,
  {
    let cell = {
      let mut services_guard = self.services.lock().expect("Poisoned lock");

      services_guard
        .entry(TypeId::of::<T>())
        .or_insert_with(|| Arc::new(OnceLock::new()))
        .clone()
    };

    let service = cell.get_or_init(|| {
      Arc::new(T::new_service(self, &self.repositories)) as Arc<dyn Any + Send + Sync>
    });

    service
      .clone()
      .downcast::<T>()
      .expect("Failed to downcast service")
  }
}