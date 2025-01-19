use libp2p::{
    gossipsub::{self, Gossipsub, GossipsubConfig, MessageAuthenticity},
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    noise::{NoiseAuthenticated, Keypair},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::{GenTcpConfig}, // Burada GenTcpConfig kullanıyoruz
    yamux,
    Multiaddr,
    NetworkBehaviour,
    PeerId,
    Transport
};
use sha2::{Sha256, Digest};
use async_std::task;
use std::error::Error;

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: Gossipsub,
    mdns: Mdns,
}

impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(peers) => {
                for peer in peers {
                    println!("Discovered peer: {:?}", peer);
                }
            },
            MdnsEvent::Expired(peers) => {
                for peer in peers {
                    println!("Expired peer: {:?}", peer);
                }
            },
        }
    }
}

impl NetworkBehaviourEventProcess<gossipsub::GossipsubEvent> for MyBehaviour {
    fn inject_event(&mut self, event: gossipsub::GossipsubEvent) {
        match event {
            gossipsub::GossipsubEvent::Message { propagation_source, message_id, message, source, topic } => {
                println!("Received message on topic {}: {:?} from {:?}", topic, message, source);
                // Here you could process the received message
            },
            _ => {}
        }
    }
}

pub async fn store_chunk_on_node(chunk_data: &[u8], node_address: &str) -> Result<String, Box<dyn Error>> {
    // 1. Compute the hash of the data
    let mut hasher = Sha256::new();
    hasher.update(chunk_data);
    let chunk_hash = format!("{:x}", hasher.finalize());

    // 2. Generate the local peer ID and key pair
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local Peer ID: {:?}", local_peer_id);

    // 3. Create the transport (TCP + Noise + Yamux)
    let transport = GenTcpConfig::new()
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(NoiseAuthenticated::xx(&local_key)?)
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    // 4. Create the network behavior
    let gossipsub_config = GossipsubConfig::default();
    let gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )?;
    let mdns = Mdns::new(MdnsConfig::default()).await?;

    let behaviour = MyBehaviour { gossipsub, mdns };

    // 5. Create the Swarm => node management system
    let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id.clone())
        .executor(Box::new(|fut| {
            task::spawn(fut);
        }))
        .build();

    // 6. Parse the remote node address
    let addr: Multiaddr = node_address.parse()?;
    Swarm::dial_addr(&mut swarm, addr.clone())?;
    println!("Dialing node at address: {:?}", addr);

    // 7. Publish the chunk using Gossipsub
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(chunk_hash.clone(), chunk_data.to_vec())?;
    println!("Data sent with hash: {}", chunk_hash);

    Ok(chunk_hash)
}
