

use mqtt;
use mqtt::packet::*;
use serde_json::{self, Result, Value};
use mqtt::{Decodable, Encodable, QualityOfService};
use mqtt::{TopicFilter, TopicName};
use std::env;
use std::io::{self, Write};
use serde_derive::{Serialize, Deserialize};
use std::io::{Error, ErrorKind};

use log::{info, warn, error, trace};

use ::futures::Future;
use mysql;
use crossbeam_channel::{bounded, tick, Sender, Receiver, select};
use crate::event_room::*;
use crate::room::User;

#[derive(Serialize, Deserialize)]
struct LoginData {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct LogoutData {
    id: String,
}


pub fn login(stream: &mut std::net::TcpStream, id: String, v: Value, pool: mysql::Pool, sender: Sender<RoomEventData>)
 -> std::result::Result<(), std::io::Error>
{
    let data: LoginData = serde_json::from_value(v).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let qres = conn.query(format!("update user set status='online' where userid='{}';", data.id));
    let publish_packet = match qres {
        Ok(_) => {
            PublishPacket::new(TopicName::new(id.clone()).unwrap(), QoSWithPacketIdentifier::Level0, "{\"msg\":\"ok\"}".to_string());
            sender.send(RoomEventData::Login(UserLoginData {u: User { id: id, ng: 1000, rk: 1000}}));
        },
        _=> {
            PublishPacket::new(TopicName::new(id.clone()).unwrap(), QoSWithPacketIdentifier::Level0, "{\"msg\":\"fail\"}".to_string());
        }
    };
    let mut buf = Vec::new();
    publish_packet.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    Ok(())
}

pub fn logout(stream: &mut std::net::TcpStream, id: String, v: Value, pool: mysql::Pool, sender: Sender<RoomEventData>)
 -> std::result::Result<(), std::io::Error>
{
    let data: LogoutData = serde_json::from_value(v).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let qres = conn.query(format!("update user set status='offline' where userid='{}';", data.id));
    let publish_packet = match qres {
        Ok(_) => {
            PublishPacket::new(TopicName::new(id.clone()).unwrap(), QoSWithPacketIdentifier::Level0, "{\"msg\":\"ok\"}".to_string());
            sender.send(RoomEventData::Logout(UserLogoutData { id: id}));
        },
        _=> {
            PublishPacket::new(TopicName::new(id.clone()).unwrap(), QoSWithPacketIdentifier::Level0, "{\"msg\":\"fail\"}".to_string());
        }
    };
    let mut buf = Vec::new();
    publish_packet.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    Ok(())
}
