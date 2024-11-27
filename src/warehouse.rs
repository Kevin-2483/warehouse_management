use base64::Engine;
use diesel::sqlite::SqliteConnection;
use diesel::insert_into;
use libp2p::identity::ed25519;
use libp2p::identity::PublicKey;
use libp2p::PeerId;
use log::info;
use crate::models::{Warehouse, NewWarehouse};
use base64::engine::general_purpose;
use crate::schema::warehouses;
use diesel::RunQueryDsl;
use diesel::prelude::*;

pub fn generate_and_insert_new_local_key(conn: &mut SqliteConnection) -> ed25519::Keypair {
    let local_key = ed25519::Keypair::generate();
    let local_key_base64 = general_purpose::STANDARD.encode(local_key.encode());
    info!(
        "Generated and inserted new key {:?} for warehouse ThisWarehouse",
        local_key_base64
    );
    let local_peer_id = PeerId::from(PublicKey::Ed25519(local_key.public()));
    let new_warehouse = NewWarehouse {
        localkey: Some(local_key_base64),
        warehouse_name: "ThisWarehouse".to_string(),
        location: "/ip4/127.0.0.1/tcp/8080".to_string(),
        capacity: Some(1000),
    };

    use self::warehouses::dsl::*;
    insert_into(warehouses)
        .values(&new_warehouse)
        .execute(conn)
        .expect("Error inserting new warehouse");

    local_key
} 

// 获取名为"ThisWarehouse"的仓库ID
pub fn get_warehouse_id(
    conn: &mut SqliteConnection,
) -> Result<ed25519::Keypair, diesel::result::Error> {
    use self::warehouses::dsl::*;
    let warehouse: Warehouse = warehouses
        .filter(warehouse_name.eq("ThisWarehouse"))
        .filter(localkey.is_not_null())
        .first(conn)?;
    info!("Found warehouse: {:?}", warehouse.localkey);
    let mut local_key_bytes = if let Some(local_key) = &warehouse.localkey {
        general_purpose::STANDARD
            .decode(local_key)
            .expect("Base64 decode error")
    } else {
        Vec::new() // 或者其他默认值的处理
    };
    let keypair =
        ed25519::Keypair::decode(local_key_bytes.as_mut_slice()).expect("Keypair decode error");
    Ok(keypair)
}