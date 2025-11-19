use sea_orm::prelude::DatabaseConnection;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait Repository: Any + Send + Sync {
  type Model;

  fn find_all(&self) -> Vec<Self::Model>;
}

pub struct Repositories {
  repositories: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
  db: DatabaseConnection,
}

impl Repositories {
  pub fn new(db: DatabaseConnection) -> Self {
    Self {
      repositories: RwLock::new(HashMap::new()),
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
}
