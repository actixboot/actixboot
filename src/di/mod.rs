use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use actix_web::web::Data;
use sea_orm::DatabaseConnection;
use crate::service::Service;

pub trait GetOrCreate {
  fn get_or_create(ctx: &DIContext) -> Data<Self>;
}

pub struct DIContext {
  repositories: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
  services: Mutex<HashMap<TypeId, Arc<OnceLock<Arc<dyn Any + Send + Sync>>>>>,
  db: DatabaseConnection,
}

impl DIContext {
  pub fn new(db: DatabaseConnection) -> Self {
    Self {
      repositories: RwLock::new(HashMap::new()),
      services: Mutex::new(HashMap::new()),
      db,
    }
  }

  pub fn get_repository<T>(&self) -> Arc<T>
  where
    T: From<DatabaseConnection> + Send + Sync + 'static,
  {
    {
      let repositories_guard = self.repositories.read().expect("Poisoned lock");

      if let Some(repository) = repositories_guard.get(&TypeId::of::<T>()) {
        return repository
          .clone()
          .downcast()
          .expect("Failed to downcast repository");
      }
    }

    let mut repositories_write_guard = self.repositories.write().expect("Poisoned lock");
    let repository = Arc::new(T::from(self.db.clone()));

    repositories_write_guard.insert(TypeId::of::<T>(), repository.clone());

    repository
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
      Arc::new(T::new_service(self)) as Arc<dyn Any + Send + Sync>
    });

    service
      .clone()
      .downcast::<T>()
      .expect("Failed to downcast service")
  }
}
