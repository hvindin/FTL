/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  FTL Information - db stats
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util::{Reply, reply_data};
use auth::User;

/// Read db stats from FTL
#[get("/settings/ftldb")]
pub fn get_ftldb(ftl: State<FtlConnectionType>, _auth: User) -> Reply {
    let mut con = ftl.connect("dbstats")?;
    // Read in FTL's database stats
    let db_queries = con.read_i32()?;
    let db_filesize = con.read_i64()?;
    let mut version_buffer = [0u8; 64];
    let db_sqlite_version = con.read_str(&mut version_buffer)?;  
    con.expect_eom()?;

    reply_data(json!({
        "queries": db_queries,
        "filesize": db_filesize,
        "sqlite_version": db_sqlite_version
    }))
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    // Basic test for reported values
    fn test_get_ftldb() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1048576).unwrap();
        encode::write_i64(&mut data, 32768).unwrap();
        encode::write_str(&mut data, "3.0.1").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/settings/ftldb")
            .ftl("dbstats", data)
            .expect_json(
                json!({
                    "queries": 1048576,
                    "filesize": 32768,
                    "sqlite_version": "3.0.1"
                })
            )
            .test();
    }

    #[test]
    // Test for (unlikely/"impossible") null report / sqlite not present
    fn test_get_ftldb_noentries() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 0).unwrap();
        encode::write_i64(&mut data, 0).unwrap();
        encode::write_str(&mut data, "").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/settings/ftldb")
            .ftl("dbstats", data)
            .expect_json(
                json!({
                    "queries": 0,
                    "filesize": 0,
                    "sqlite_version": ""
                })
            )
            .test();
    }
}
