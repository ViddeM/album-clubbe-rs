use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

fn main() {
    let mut args = std::env::args();
    let program_name = args
        .next()
        .unwrap_or_else(|| "hash_admin_token".to_string());

    let Some(token) = args.next() else {
        eprintln!("Usage: {program_name} <plain-admin-token>");
        std::process::exit(2);
    };

    if token.trim().is_empty() {
        eprintln!("Token must not be empty");
        std::process::exit(2);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = match argon2.hash_password(token.as_bytes(), &salt) {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("Failed to hash token: {error}");
            std::process::exit(1);
        }
    };

    println!("{}", hash);
}
