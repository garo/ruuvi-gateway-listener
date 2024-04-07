use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::time::Duration;
use lazy_static::lazy_static;
use prometheus::{Counter, Encoder, Gauge, HistogramVec, TextEncoder};
use prometheus::{labels, opts, register_counter, register_gauge, register_histogram_vec};

use rumqttc::{MqttOptions, AsyncClient, QoS};
use rumqttc::Event::Incoming;
use rumqttc::Packet::{Publish, ConnAck, SubAck, PingResp};
//use std::{env, process, thread};
mod ruuvi;

use crate::ruuvi::gateway::GatewayMessageResult;

lazy_static! {
    static ref HTTP_COUNTER: Counter = register_counter!(opts!(
        "ruuvi_http_requests_total",
        "Number of HTTP requests made.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
}

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    HTTP_COUNTER.inc();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() {
    // Setup paho mqtt

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "mqtt.juhonkoti.net", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("ruuvi/#", QoS::AtMostOnce).await.unwrap();

    

    // Setup Prometheus
    let addr = ([0, 0, 0, 0], 9898).into();
    println!("Listening on http://{}", addr);

    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(serve_req))
    }));

    println!("Preparing to serve prometheus traffic...");
    tokio::spawn(async move { 
        if let Err(err) = serve_future.await {
            eprintln!("server error: {}", err);
        }    
    });

    
    println!("Starting event loop polling");
    let mut sink = ruuvi::prometheus::RuuviPrometheusSink::new();

    
    tokio::spawn(async move {
        for i in 0..1000 {
            //client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
            println!("Timer 1000 ms");
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });

    while let Ok(notification) = eventloop.poll().await {
        match notification {
            Incoming(incoming) => {
                match incoming {
                    Publish(publish) => {
                        //println!("Incoming message to topic {:?}, message: {:?}", publish.topic, publish.payload);
                        let message_result = ruuvi::gateway::parse_gateway_message(&publish.payload, publish.topic);
                        match message_result {
                            GatewayMessageResult::Received(message) => {
                                println!("message: {:?}", message);
                                ruuvi::parser::decode_ble_ruuvi_str(&message.data, &message.mac, &mut sink);
                            }
                            _ => {}
                        }
                    }

                    ConnAck(connack) => {
                        println!("Connection acknowledged: {:?}", connack.code)
                    }

                    SubAck(suback) => {
                        println!("Subscription acknowledged: {:?}", suback.return_codes)
                    }

                    PingResp => {}

                    _ => {
                        println!("Received something else = {:?}", incoming);
                    }
                }
            }
            _ => {}
        }
    }

}
