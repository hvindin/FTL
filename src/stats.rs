use util;
use ftl;

use std::collections::HashMap;
use rmp::decode::{ValueReadError, DecodeStringError};
use rmp::Marker;

#[derive(Serialize)]
struct Query(i32, String, String, String, u8, u8);

#[get("/stats/summary")]
pub fn summary() -> util::Reply {
    let mut con = ftl_connect!("stats");

    let domains_blocked = con.read_i32().unwrap();
    let total_queries = con.read_i32().unwrap();
    let blocked_queries = con.read_i32().unwrap();
    let percent_blocked = con.read_f32().unwrap();
    let unique_domains = con.read_i32().unwrap();
    let forwarded_queries = con.read_i32().unwrap();
    let cached_queries = con.read_i32().unwrap();
    let total_clients = con.read_i32().unwrap();
    let unique_clients = con.read_i32().unwrap();
    let status = con.read_u8().unwrap();
    con.expect_eom().unwrap();

    util::reply_data(json!({
        "domains_blocked": domains_blocked,
        "total_queries": total_queries,
        "blocked_queries": blocked_queries,
        "percent_blocked": percent_blocked,
        "unique_domains": unique_domains,
        "forwarded_queries": forwarded_queries,
        "cached_queries": cached_queries,
        "total_clients": total_clients,
        "unique_clients": unique_clients,
        "status": status
    }))
}

#[get("/stats/overTime")]
pub fn over_time() -> util::Reply {
    let mut con = ftl_connect!("overTime");

    let domains_over_time = con.read_int_map().unwrap();
    let blocked_over_time = con.read_int_map().unwrap();

    util::reply_data(json!({
        "domains_over_time": domains_over_time,
        "blocked_over_time": blocked_over_time
    }))
}

fn get_top_domains(blocked: bool) -> util::Reply {
    let command = if blocked { "top-ads" } else { "top-domains" };

    let mut con = ftl_connect!(command);
    let queries = con.read_i32().unwrap();

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut top: HashMap<String, i32> = HashMap::new();

    loop {
        let domain = match con.read_str(&mut str_buffer) {
            Ok(domain) => domain,
            Err(e) => {
                if let DecodeStringError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        let count = con.read_i32().unwrap();

        top.insert(domain.to_string(), count);
    }

    let (top_type, queries_type) = if blocked {
        ("top_ads", "blocked_queries")
    } else {
        ("top_domains", "total_queries")
    };

    util::reply_data(json!({
        top_type: top,
        queries_type: queries
    }))
}

#[get("/stats/top_domains")]
pub fn top_domains() -> util::Reply {
    get_top_domains(false)
}

#[get("/stats/top_blocked")]
pub fn top_blocked() -> util::Reply {
    get_top_domains(true)
}

#[get("/stats/top_clients")]
pub fn top_clients() -> util::Reply {
    let mut con = ftl_connect!("top-clients");
    let total_queries = con.read_i32().unwrap();

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut top_clients: HashMap<String, i32> = HashMap::new();

    loop {
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_string(),
            Err(e) => {
                if let DecodeStringError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        let ip = con.read_str(&mut str_buffer).unwrap();
        let count = con.read_i32().unwrap();

        let key = if ip.len() > 0 {
            format!("{}|{}", name, ip)
        } else {
            name
        };

        top_clients.insert(key, count);
    }

    util::reply_data(json!({
        "top_clients": top_clients,
        "total_queries": total_queries
    }))
}

#[get("/stats/history")]
pub fn history() -> util::Reply {
    let mut con = ftl_connect!("getallqueries");

    let mut history: Vec<Query> = Vec::new();

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    loop {
        let timestamp = match con.read_i32() {
            Ok(timestamp) => timestamp,
            Err(e) => {
                if let ValueReadError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        let query_type = con.read_str(&mut str_buffer).unwrap().to_owned();
        let domain = con.read_str(&mut str_buffer).unwrap().to_owned();
        let client = con.read_str(&mut str_buffer).unwrap().to_owned();
        let status = con.read_u8().unwrap();
        let dnssec = con.read_u8().unwrap();

        history.push(Query(timestamp, query_type, domain, client, status, dnssec));
    }

    util::reply_data(json!({
        "history": history
    }))
}
