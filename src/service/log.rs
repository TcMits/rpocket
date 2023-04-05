use crate::service;

pub struct LogService<'a, C> {
    client: &'a mut C,
    request_base_path: String,
}

impl<'a, C> LogService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new LogService.
    pub fn new(client: &'a mut C) -> Self {
        return LogService {
            client,
            request_base_path: "api/logs/requests".to_string(),
        };
    }

    /// returns crud service.
    pub fn crud(&'a mut self) -> service::CRUDService<'a, C> {
        return self.client.crud(&self.request_base_path);
    }
}

#[cfg(test)]
mod test {
    use crate::PocketBase;

    use super::*;

    #[test]
    fn test_record_crud() {
        let mut base = PocketBase::new("http://test.com", "en");
        let mut record_service = LogService::new(&mut base);
        let crud = record_service.crud();

        assert!(crud.base_path == "api/logs/requests");
    }
}
