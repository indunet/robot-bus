use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use prost::Message;

use crate::console_topics;
use crate::errors::Result;
use crate::robot_bus_interfaces::msg::v1::{TopicDemand, TopicStatsList};
use crate::ros2_bridge::mapper::{
    ActionWireContext, Direction, ServiceWireContext, TopicMapper, TopicQos, TopicWireContext,
};
use crate::runtime::{MessageCallback, Node, QosProfile, SubscriptionHandle, TopicPublisherRaw};

use super::specs::{ActionRouteSpec, DemandEvent, LazyRos2ToBus, RouteSpec, ServiceRouteSpec};

pub(super) fn create_ros2_to_bus_publisher(
    bus_node: &mut Node,
    bus_topic: &str,
    qos: TopicQos,
) -> Result<TopicPublisherRaw> {
    let pub_ =
        bus_node.create_publisher_raw_with_qos(bus_topic, QosProfile::keep_last(qos.depth()))?;
    if let Err(e) = pub_.set_send_timeout_ms(0) {
        log::warn!("ros→bus {bus_topic} send timeout: {e}");
    }
    Ok(pub_)
}

pub(super) fn create_ros2_to_bus_sub(
    ros_node: &rclrs::Node,
    bus_pub: TopicPublisherRaw,
    mapper: &Arc<dyn TopicMapper>,
    ros_topic: &str,
    qos: TopicQos,
) -> Result<Box<dyn Any + Send + Sync>> {
    mapper.create_ros2_to_bus_subscription(ros_node, bus_pub, ros_topic, qos)
}

pub(super) fn subscribe_demand(
    bus_node: &mut Node,
    demand_tx: Sender<DemandEvent>,
) -> Result<Vec<SubscriptionHandle>> {
    let tx_demand = demand_tx.clone();
    let demand_cb: MessageCallback =
        Arc::new(move |_topic, payload| match TopicDemand::decode(payload) {
            Ok(msg) => {
                let _ = tx_demand.send(DemandEvent::Count {
                    topic: msg.topic,
                    subscribers: msg.subscribers,
                });
            }
            Err(err) => log::warn!("decode TopicDemand: {err}"),
        });
    let h1 = bus_node.create_subscription_raw(console_topics::TOPIC_DEMAND, demand_cb, None)?;

    let tx_topics = demand_tx;
    let topics_cb: MessageCallback = Arc::new(move |_topic, payload| match TopicStatsList::decode(
        payload,
    ) {
        Ok(list) => {
            let counts = list
                .topics
                .into_iter()
                .map(|t| (t.name, t.subscribers as u32))
                .collect();
            let _ = tx_topics.send(DemandEvent::Snapshot { counts });
        }
        Err(err) => log::warn!("decode TopicStatsList: {err}"),
    });
    let h2 = bus_node.create_subscription_raw(console_topics::TOPICS, topics_cb, None)?;
    Ok(vec![h1, h2])
}

pub(super) fn wire_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    bus_pubs: &mut HashMap<String, TopicPublisherRaw>,
    lazy_routes: &mut HashMap<String, LazyRos2ToBus>,
    eager_bus_topics: &mut HashSet<String>,
    route: &RouteSpec,
    ros_subs: &mut Vec<Box<dyn Any + Send + Sync>>,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    let mapper = Arc::clone(&route.mapper);
    let ros_topic = route.ros_topic.clone();
    let bus_topic = route.bus_topic.clone();
    let ros_qos = route.ros_qos;
    let bus_qos = route.bus_qos;

    match route.direction {
        Direction::BusToRos2 => {
            mapper.attach_bus_to_ros(TopicWireContext {
                ros_node,
                bus_node,
                ros_topic: ros_topic.as_str(),
                bus_topic: bus_topic.as_str(),
                ros_qos,
                bus_qos,
                ros_entities,
            })?;
        }
        Direction::Ros2ToBus => {
            let bus_pub = create_ros2_to_bus_publisher(bus_node, bus_topic.as_str(), bus_qos)?;
            bus_pubs.insert(bus_topic.clone(), bus_pub.clone());
            if route.lazy {
                lazy_routes.insert(
                    bus_topic,
                    LazyRos2ToBus {
                        ros_topic,
                        mapper,
                        ros_qos,
                        sub: None,
                    },
                );
            } else {
                let sub = create_ros2_to_bus_sub(ros_node, bus_pub, &mapper, &ros_topic, ros_qos)?;
                ros_subs.push(sub);
                eager_bus_topics.insert(bus_topic);
            }
        }
    }

    Ok(())
}

pub(super) fn wire_service_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    route: &ServiceRouteSpec,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    route.mapper.attach(ServiceWireContext {
        ros_node,
        bus_node,
        ros_service: route.ros_service.as_str(),
        bus_service: route.bus_service.as_str(),
        direction: route.direction,
        timeout: route.timeout,
        ros_qos: route.ros_qos,
        bus_qos: route.bus_qos,
        ros_entities,
    })
}

pub(super) fn wire_action_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    route: &ActionRouteSpec,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    route.mapper.attach(ActionWireContext {
        ros_node,
        bus_node,
        ros_action: route.ros_action.as_str(),
        bus_action: route.bus_action.as_str(),
        direction: route.direction,
        timeout: route.timeout,
        ros_qos: route.ros_qos,
        bus_qos: route.bus_qos,
        ros_entities,
    })
}
