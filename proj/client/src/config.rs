//! Строки подключения к центру и объекту КИИ.
//!
//! Значения можно переопределить переменными окружения `RBD_CENTER`
//! и `RBD_SITE`; если они не заданы, используются локальные значения
//! по умолчанию, рассчитанные на демонстрационный docker-compose стенд.

use std::env;

const DEFAULT_CENTER_URL: &str = "postgres://center:center_pass@127.0.0.1:5432/rbd_center";
const DEFAULT_SITE_URL: &str = "postgres://site:site_pass@127.0.0.1:5433/rbd_site";

/// Пара адресов подключения: центр и объект КИИ.
pub struct Endpoints {
    pub center: String,
    pub site: String,
}

impl Endpoints {
    /// Читает адреса из окружения, подставляя значения по умолчанию.
    pub fn from_env() -> Self {
        let center = env::var("RBD_CENTER").unwrap_or_else(|_| DEFAULT_CENTER_URL.to_owned());
        let site = env::var("RBD_SITE").unwrap_or_else(|_| DEFAULT_SITE_URL.to_owned());
        Endpoints { center, site }
    }

    pub fn masked_center(&self) -> String {
        mask_credentials(&self.center)
    }

    pub fn masked_site(&self) -> String {
        mask_credentials(&self.site)
    }
}

/// Скрывает пароль в строке подключения вида `scheme://user:pass@host/db`,
/// оставляя видимыми схему, имя пользователя и хост.
pub fn mask_credentials(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_owned();
    };
    let Some((credentials, host_and_db)) = rest.split_once('@') else {
        return url.to_owned();
    };
    let Some((user, _password)) = credentials.split_once(':') else {
        return url.to_owned();
    };
    format!("{scheme}://{user}:***@{host_and_db}")
}

#[cfg(test)]
mod tests {
    use super::mask_credentials;

    #[test]
    fn hides_password_but_keeps_user_and_host() {
        let masked = mask_credentials("postgres://center:center_pass@127.0.0.1:5432/rbd_center");
        assert_eq!(masked, "postgres://center:***@127.0.0.1:5432/rbd_center");
    }

    #[test]
    fn returns_input_unchanged_when_format_is_unexpected() {
        let masked = mask_credentials("not-a-url");
        assert_eq!(masked, "not-a-url");
    }
}
