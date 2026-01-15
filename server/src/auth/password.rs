use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, Params};
use rand::rngs::OsRng;

const MIN_PASSWORD_LEN: usize = 12;

fn argon2() -> Argon2<'static> {
    // Balanced defaults for a small self-hosted server.
    // - Memory: 19 MiB
    // - Iterations: 2
    // - Parallelism: 1
    // Adjust later if needed.
    let params = Params::new(19 * 1024, 2, 1, None).expect("argon2 params");
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
}

pub fn validate_new_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        ));
    }
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, String> {
    validate_new_password(password)?;
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| "Failed to hash password".to_string())?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password_hash: &str, password: &str) -> Result<bool, String> {
    let parsed =
        PasswordHash::new(password_hash).map_err(|_| "Invalid password hash".to_string())?;
    Ok(argon2()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
