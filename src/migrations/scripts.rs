use crate::permissions;
use futures::StreamExt;
use log::info;
use mongodb::{
    bson::{doc, from_document, to_document},
    options::FindOptions,
    Database,
};
use serde::{Deserialize, Serialize};

pub type PermissionTuple = (
    i32, // server permission
    i32, // channel permission
);

#[derive(Serialize, Deserialize)]
struct MigrationInfo {
    _id: i32,
    revision: i32,
}

pub const LATEST_REVISION: i32 = 8;

pub async fn migrate_database(db: &Database) {
    let migrations = db.collection("migrations");
    let data = migrations
        .find_one(None, None)
        .await
        .expect("Failed to fetch migration data.");

    if let Some(doc) = data {
        let info: MigrationInfo =
            from_document(doc).expect("Failed to read migration information.");

        let revision = run_migrations(db, info.revision).await;

        migrations
            .update_one(
                doc! {
                    "_id": info._id
                },
                doc! {
                    "$set": {
                        "revision": revision
                    }
                },
                None,
            )
            .await
            .expect("Failed to commit migration information.");

        info!("Migration complete. Currently at revision {}.", revision);
    } else {
        panic!("Database was configured incorrectly, possibly because initalization failed.")
    }
}

pub async fn run_migrations(db: &Database, revision: i32) -> i32 {
    info!("Starting database migration.");

    if revision <= 0 {
        info!("Running migration [revision 0]: Test migration system.");
    }

    if revision <= 1 {
        info!("Running migration [revision 1 / 2021-04-24]: Migrate to Autumn v1.0.0.");

        let messages = db.collection("messages");
        let attachments = db.collection("attachments");

        messages
            .update_many(
                doc! { "attachment": { "$exists": 1 } },
                doc! { "$set": { "attachment.tag": "attachments", "attachment.size": 0 } },
                None,
            )
            .await
            .expect("Failed to update messages.");

        attachments
            .update_many(
                doc! {},
                doc! { "$set": { "tag": "attachments", "size": 0 } },
                None,
            )
            .await
            .expect("Failed to update attachments.");
    }

    if revision <= 2 {
        info!("Running migration [revision 2 / 2021-05-08]: Add servers collection.");

        db.create_collection("servers", None)
            .await
            .expect("Failed to create servers collection.");
    }

    if revision <= 3 {
        info!("Running migration [revision 3 / 2021-05-25]: Support multiple file uploads, add channel_unreads and user_settings.");

        let messages = db.collection("messages");
        let mut cursor = messages
            .find(
                doc! {
                    "attachment": {
                        "$exists": 1
                    }
                },
                FindOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "attachments": [ "$attachment" ]
                    })
                    .build(),
            )
            .await
            .expect("Failed to fetch messages.");

        while let Some(result) = cursor.next().await {
            let doc = result.unwrap();
            let id = doc.get_str("_id").unwrap();
            let attachments = doc.get_array("attachments").unwrap();

            messages
                .update_one(
                    doc! { "_id": id },
                    doc! { "$unset": { "attachment": 1 }, "$set": { "attachments": attachments } },
                    None,
                )
                .await
                .unwrap();
        }

        db.create_collection("channel_unreads", None)
            .await
            .expect("Failed to create channel_unreads collection.");

        db.create_collection("user_settings", None)
            .await
            .expect("Failed to create user_settings collection.");
    }

    if revision <= 4 {
        info!("Running migration [revision 4 / 2021-06-01]: Add more server collections.");

        db.create_collection("server_members", None)
            .await
            .expect("Failed to create server_members collection.");

        db.create_collection("server_bans", None)
            .await
            .expect("Failed to create server_bans collection.");

        db.create_collection("channel_invites", None)
            .await
            .expect("Failed to create channel_invites collection.");
    }

    if revision <= 5 {
        info!("Running migration [revision 5 / 2021-06-26]: Add permissions.");

        #[derive(Serialize)]
        struct Server {
            pub default_permissions: PermissionTuple,
        }

        let server = Server {
            default_permissions: (
                *permissions::server::DEFAULT_PERMISSION as i32,
                *permissions::channel::DEFAULT_PERMISSION_SERVER as i32,
            ),
        };

        db.collection("servers")
            .update_many(
                doc! {},
                doc! {
                    "$set": to_document(&server).unwrap()
                },
                None,
            )
            .await
            .expect("Failed to migrate servers.");
    }

    if revision <= 6 {
        info!("Running migration [revision 6 / 2021-07-09]: Add message text index.");

        db.run_command(
            doc! {
                "createIndexes": "messages",
                "indexes": [
                    {
                        "key": {
                            "content": "text"
                        },
                        "name": "content"
                    }
                ]
            },
            None,
        )
        .await
        .expect("Failed to create message index.");
    }

    if revision <= 7 {
        info!("Running migration [revision 7 / 2021-08-11]: Add message text index.");

        db.create_collection("bots", None)
            .await
            .expect("Failed to create bots collection.");
    }

    // Reminder to update LATEST_REVISION when adding new migrations.
    LATEST_REVISION
}
