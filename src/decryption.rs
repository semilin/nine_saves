use crate::save::{Save, SaveInfo};

use anyhow::Result;
use base64::prelude::*;
use libaes::Cipher;
use sonic_rs;
use std::fs;

const KEY: [u8; 16] = *b"1234567812345678";

impl Save {
    fn encrypted_metadata(&self) -> Result<Vec<u8>> {
        Ok(fs::read(self.path.join("meta.txt"))?)
    }
    #[allow(dead_code)]
    fn encrypted_flags(&self) -> Result<Vec<u8>> {
        Ok(fs::read(self.path.join("flags.txt"))?)
    }
}

fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
    let ciphertext = BASE64_STANDARD.decode(&data)?;
    let cipher = Cipher::new_128(&KEY);
    let decrypted = cipher.cbc_decrypt(&KEY, &ciphertext[..]);
    Ok(decrypted)
}

impl SaveInfo {
    pub fn decrypt_from(save: &Save) -> Result<Self> {
        let data = save.encrypted_metadata()?;
        let decrypted = decrypt(&data)?;
        let info: SaveInfo = sonic_rs::from_slice(&decrypted)?;

        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decryption() {
        let data = "1s5HvVtuLGW3T+UEb8oZh7+PEknj0CgMijOHVWILDy7hC9ah92LSPHX5OKzY4LOg9P3IpdGzGKMZfUWOc/sfEiYS2MsXRICTdDV/yfAZI6ovIMZjI6BiS7vwqcsL3lbEPkUbP9HBJC+agWNdh6F/c+9zUlrq6hDbGCyNpgMYc/Bmnz2aaOK3eYtqxW7YNQ9GiN23YtLAet0SEHYaUk/oE1HWXQIfZiXjN6DLJwkwWq6C4q6LfoZo6Ss99aF20qo9s5MkEXhaoRIXBWcxvM8GamkCTES8mBduzpmFDZtHQn27UVSmhLNbfh9oBs5WjeErhZABDRQqTzTtCvK8YI5DbUXupX2mFomp68sOHoxGegw45tR7hLZdl8lc7aP+XxZR8FpTRLnCKcw1uSozHP3qQZvC5tLTx7QH8A9CSh4wG8iwMs/06qIwqLCKJznn/+wDUY5X/yfUafV5pexXm/LRkUjKuV+igzc6ULT6zPLdRogobvqeKruAIxUPibG7mfjXYXs37Jw4Ld111lJxP16X5gFZixcTbqG6IMg/SqegyZ6RZ3yrpxv7g54WsMq7tzuJ".as_bytes();
        let decrypted = decrypt(&data).unwrap();
    }
}
