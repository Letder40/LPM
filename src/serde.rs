use serde_derive::{Deserialize, Serialize};
use bincode::{deserialize, serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct PasswordData {
    pub id: String,
    pub value: String
}

pub fn serialize_passwords(passfile_data: &Vec<PasswordData>) -> Vec<u8>{
    let passfile_data_bytes = serialize(&passfile_data).unwrap();
    passfile_data_bytes
}

pub fn deserialize_passwords(passfile_data_bytes: &Vec<u8>) -> Vec<PasswordData>{
    let passfile_data:Vec<PasswordData> = deserialize(&passfile_data_bytes).unwrap();
    passfile_data
}