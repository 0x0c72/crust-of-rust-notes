use redis::{self, FromRedisValue, Commands};
use serde_json::{self, from_slice, Value};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct My {
    s: String,
    x: i16,
}

impl FromRedisValue for My {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let v: Vec<u8> = redis::from_redis_value(v)?;
        Ok(serde_json::from_slice(v.as_slice()).expect("!json"))
    }
}

#[derive(Debug)]
struct Zy {
    a: i32,
    b: String,
}

impl FromRedisValue for Zy {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let v: std::collections::HashMap<i32, String> = redis::from_redis_value(v)?;
        let a = v.keys().next().unwrap();
        let b = &v[a];
        Ok(Zy {
            a: *a,
            b: b.to_string(),
        })
    }

    fn from_redis_values(items: &[redis::Value]) -> redis::RedisResult<Vec<Self>> {
        Ok(items
            .iter()
            .filter_map(|item| FromRedisValue::from_redis_value(item).ok())
            .collect())
    }

    fn from_byte_vec(_vec: &[u8]) -> Option<Vec<Self>> {
        None
    }
}

fn main() {
    let redis_url = std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into());
    let client = redis::Client::open(redis_url).unwrap();
    let mut conn = client.get_connection().expect("!conn");
    let my = My {
        s: "1".to_string(),
        x: 10,
    };
    let json = serde_json::to_string(&my).unwrap();
    let value = json.as_bytes();
    println!("{:?}", value);
    let _: () = conn.set("key1", value).unwrap();
    let x: My = conn.get("key1").unwrap();
    println!("{:?}", x);

    let items = [(10, "dten"), (20, "etwenty")];
    let _: () = conn.hset_multiple("key2", &items).unwrap();
    let y: String = conn.hget("key2", 10).unwrap();
    println!("{:?}", y);
    let z: Zy = conn.hgetall("key2").unwrap();
        
    println!("{:?}", z);
}