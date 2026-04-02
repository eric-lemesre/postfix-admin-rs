mod admin_repository;
mod alias_domain_repository;
mod alias_repository;
mod app_password_repository;
mod dkim_repository;
mod domain_repository;
mod fetchmail_repository;
mod log_repository;
mod mailbox_repository;
mod vacation_repository;

pub use admin_repository::AdminRepository;
pub use alias_domain_repository::AliasDomainRepository;
pub use alias_repository::AliasRepository;
pub use app_password_repository::AppPasswordRepository;
pub use dkim_repository::DkimRepository;
pub use domain_repository::DomainRepository;
pub use fetchmail_repository::FetchmailRepository;
pub use log_repository::LogRepository;
pub use mailbox_repository::MailboxRepository;
pub use vacation_repository::VacationRepository;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn assert_object_safe<T: ?Sized>() {}

    #[test]
    fn domain_repository_is_object_safe() {
        assert_object_safe::<dyn DomainRepository>();
    }

    #[test]
    fn mailbox_repository_is_object_safe() {
        assert_object_safe::<dyn MailboxRepository>();
    }

    #[test]
    fn alias_repository_is_object_safe() {
        assert_object_safe::<dyn AliasRepository>();
    }

    #[test]
    fn admin_repository_is_object_safe() {
        assert_object_safe::<dyn AdminRepository>();
    }

    #[test]
    fn vacation_repository_is_object_safe() {
        assert_object_safe::<dyn VacationRepository>();
    }

    #[test]
    fn dkim_repository_is_object_safe() {
        assert_object_safe::<dyn DkimRepository>();
    }

    #[test]
    fn fetchmail_repository_is_object_safe() {
        assert_object_safe::<dyn FetchmailRepository>();
    }

    #[test]
    fn log_repository_is_object_safe() {
        assert_object_safe::<dyn LogRepository>();
    }

    #[test]
    fn app_password_repository_is_object_safe() {
        assert_object_safe::<dyn AppPasswordRepository>();
    }

    #[test]
    fn alias_domain_repository_is_object_safe() {
        assert_object_safe::<dyn AliasDomainRepository>();
    }

    // Verify that Arc<dyn Repo> compiles (used in dependency injection)
    fn _accepts_arc_domain(_repo: Arc<dyn DomainRepository>) {}
    fn _accepts_arc_mailbox(_repo: Arc<dyn MailboxRepository>) {}
    fn _accepts_arc_alias(_repo: Arc<dyn AliasRepository>) {}
    fn _accepts_arc_admin(_repo: Arc<dyn AdminRepository>) {}
    fn _accepts_arc_vacation(_repo: Arc<dyn VacationRepository>) {}
    fn _accepts_arc_dkim(_repo: Arc<dyn DkimRepository>) {}
    fn _accepts_arc_fetchmail(_repo: Arc<dyn FetchmailRepository>) {}
    fn _accepts_arc_log(_repo: Arc<dyn LogRepository>) {}
    fn _accepts_arc_app_password(_repo: Arc<dyn AppPasswordRepository>) {}
    fn _accepts_arc_alias_domain(_repo: Arc<dyn AliasDomainRepository>) {}
}
