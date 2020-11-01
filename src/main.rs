extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_ws_server;

use std::sync::Arc;
use std::{thread, time};

use jsonrpc_core::*;
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
use jsonrpc_ws_server::{RequestContext, ServerBuilder};

use serde_json::{Map, Value};

fn main() {
    let mut io = PubSubHandler::new(MetaIoHandler::default());

    io.add_subscription(
        "channelMessage",
        (
            "test_executions",
            |params: Params, _, subscriber: Subscriber| {
                if params != Params::None {
                    return;
                }

                thread::spawn(move || {
                    let sink = subscriber.assign_id(SubscriptionId::Number(5)).unwrap();

                    loop {
                        thread::sleep(time::Duration::from_millis(1000));

                        let mut json = Map::new();
                        json.insert(String::from("channel"), Value::Null);
                        json.insert(String::from("message"), Value::Null);
                        match sink.notify(Params::Map(json)) {
                            Err(err) => {
                                println!("Subscription has ended, finishing. {}", err);
                                break;
                            },
                            _ => continue,
                        }
                    }
                });
            },
        ),
        (
            "test_executions_remove",
            |_id: SubscriptionId, _meta| -> BoxFuture<Result<Value>> {
                println!("Closing subscription");
                Box::pin(futures::future::ready(Ok(Value::Bool(true))))
            },
        ),
    );

    let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
        Arc::new(Session::new(context.sender()))
    })
    .start(&"127.0.0.1:3030".parse().unwrap())
    .expect("Unable to start RPC server");

    let _ = server.wait();
}
