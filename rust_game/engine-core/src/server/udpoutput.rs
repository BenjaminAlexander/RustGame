use crate::gametime::TimeMessage;
use crate::interface::{
    GameFactoryTrait,
    GameTrait,
};
use crate::messaging::{
    Fragmenter,
    InputMessage,
    ServerInputMessage,
    StateMessage,
    ToClientMessageUDP,
};
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::server::udpoutput::UdpOutputEvent::{
    RemotePeer,
    SendCompletedStep,
    SendInputMessage,
    SendServerInputMessage,
    SendTimeMessage,
};
use commons::factory::FactoryTrait;
use commons::net::{
    UdpSocket,
    MAX_UDP_DATAGRAM_SIZE,
};
use commons::stats::RollingAverage;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use commons::time::{
    TimeDuration,
    TimeValue,
};
use log::{
    debug,
    error,
    info,
    warn,
};
use std::io;
use std::ops::Add;

pub enum UdpOutputEvent<Game: GameTrait> {
    RemotePeer(RemoteUdpPeer),
    SendTimeMessage(TimeMessage),
    SendInputMessage(InputMessage<Game>),
    SendServerInputMessage(ServerInputMessage<Game>),
    SendCompletedStep(StateMessage<Game>),
}

pub struct UdpOutput<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    player_index: usize,
    socket: UdpSocket,
    remote_peer: Option<RemoteUdpPeer>,
    fragmenter: Fragmenter,
    last_time_message: Option<TimeMessage>,
    last_state_sequence: Option<usize>,

    //metrics
    time_in_queue_rolling_average: RollingAverage,
    time_of_last_state_send: TimeValue,
    time_of_last_input_send: TimeValue,
    time_of_last_server_input_send: TimeValue,
}

impl<GameFactory: GameFactoryTrait> UdpOutput<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        player_index: usize,
        socket: &UdpSocket,
    ) -> io::Result<Self> {
        Ok(UdpOutput {
            player_index,
            remote_peer: None,
            //TODO: move clone outside
            socket: socket.try_clone()?,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            last_time_message: None,
            last_state_sequence: None,

            //metrics
            time_in_queue_rolling_average: RollingAverage::new(100),
            time_of_last_state_send: factory.get_time_source().now(),
            time_of_last_input_send: factory.get_time_source().now(),
            time_of_last_server_input_send: factory.get_time_source().now(),

            factory,
        })
    }

    fn on_remote_peer(&mut self, remote_peer: RemoteUdpPeer) -> EventHandleResult<Self> {
        //TODO: could this be checked before calling udpoutput?
        if self.player_index == remote_peer.get_player_index() {
            info!("Setting remote peer: {:?}", remote_peer);
            self.remote_peer = Some(remote_peer);
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_completed_step(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        state_message: StateMessage<GameFactory::Game>,
    ) -> EventHandleResult<Self> {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        if self.last_state_sequence.is_none()
            || self.last_state_sequence.as_ref().unwrap() <= &state_message.get_sequence()
        {
            self.last_state_sequence = Some(state_message.get_sequence());
            self.time_of_last_state_send = self.factory.get_time_source().now();

            let message = ToClientMessageUDP::<GameFactory::Game>::StateMessage(state_message);
            self.send_message(message);

            //info!("state_message");
            self.log_time_in_queue(*time_in_queue);
        }

        return EventHandleResult::TryForNextEvent;
    }

    pub fn on_time_message(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        time_message: TimeMessage,
    ) -> EventHandleResult<Self> {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        let mut send_it = false;

        if let Some(last_time_message) = &self.last_time_message {
            if time_message.get_scheduled_time().is_after(
                &last_time_message
                    .get_scheduled_time()
                    .add(&GameFactory::Game::TIME_SYNC_MESSAGE_PERIOD),
            ) {
                send_it = true;
            }
        } else {
            send_it = true;
        }

        if send_it {
            self.last_time_message = Some(time_message.clone());

            //TODO: timestamp when the time message is set, then use that info in client side time calc
            let message = ToClientMessageUDP::<GameFactory::Game>::TimeMessage(time_message);

            self.send_message(message);

            //info!("time_message");
            self.log_time_in_queue(*time_in_queue);
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_input_message(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        input_message: InputMessage<GameFactory::Game>,
    ) -> EventHandleResult<Self> {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        if self.player_index != input_message.get_player_index()
            && (self.last_state_sequence.is_none()
                || self.last_state_sequence.as_ref().unwrap() <= &input_message.get_step())
        {
            self.time_of_last_input_send = self.factory.get_time_source().now();

            let message = ToClientMessageUDP::<GameFactory::Game>::InputMessage(input_message);
            self.send_message(message);

            //info!("input_message");
            self.log_time_in_queue(*time_in_queue);
        } else {
            //info!("InputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
        }

        return EventHandleResult::TryForNextEvent;
    }

    pub fn on_server_input_message(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        server_input_message: ServerInputMessage<GameFactory::Game>,
    ) -> EventHandleResult<Self> {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        if self.last_state_sequence.is_none()
            || self.last_state_sequence.as_ref().unwrap() <= &server_input_message.get_step()
        {
            self.time_of_last_server_input_send = self.factory.get_time_source().now();

            let message =
                ToClientMessageUDP::<GameFactory::Game>::ServerInputMessage(server_input_message);
            self.send_message(message);

            //info!("server_input_message");
            self.log_time_in_queue(*time_in_queue);
        } else {
            //info!("ServerInputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
        }

        return EventHandleResult::TryForNextEvent;
    }

    //TODO: generalize this for all channels
    fn log_time_in_queue(&mut self, time_in_queue: TimeValue) {
        let now = self.factory.get_time_source().now();
        let duration_in_queue = now.duration_since(&time_in_queue);

        self.time_in_queue_rolling_average
            .add_value(duration_in_queue.as_secs_f64());
        let average = self.time_in_queue_rolling_average.get_average();

        if average > 500.0 {
            warn!(
                "High average duration in queue: {:?} in milliseconds",
                average
            );
        }
    }

    fn send_message(&mut self, message: ToClientMessageUDP<GameFactory::Game>) {
        if let Some(remote_peer) = &self.remote_peer {
            let buf = rmp_serde::to_vec(&message).unwrap();
            let fragments = self.fragmenter.make_fragments(buf);

            for fragment in fragments {
                if fragment.get_whole_buf().len() > MAX_UDP_DATAGRAM_SIZE {
                    error!(
                        "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                        fragment.get_whole_buf().len()
                    );
                }

                self.socket
                    .send_to(fragment.get_whole_buf(), &remote_peer.get_socket_addr())
                    .unwrap();
            }
        }
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for UdpOutput<GameFactory> {
    type Event = UdpOutputEvent<GameFactory::Game>;
    type ThreadReturn = ();

    fn on_channel_event(
        &mut self,
        channel_event: ChannelEvent<Self::Event>,
    ) -> EventHandleResult<Self> {
        let now = self.factory.get_time_source().now();

        // let duration_since_last_input = now.duration_since(self.time_of_last_input_send);
        // if duration_since_last_input > TimeDuration::one_second() {
        //     warn!("It has been {:?} since last input message was sent. Now: {:?}, Last: {:?}, Queue length: {:?}",
        //           duration_since_last_input, now, self.time_of_last_input_send, self.input_queue.len());
        // }

        let duration_since_last_state = now.duration_since(&self.time_of_last_state_send);
        if duration_since_last_state > TimeDuration::ONE_SECOND {
            //TODO: this should probably be a warn when it happens less often
            debug!(
                "It has been {:?} since last state message was sent. Now: {:?}, Last: {:?}",
                duration_since_last_state, now, self.time_of_last_state_send
            );
        }

        match channel_event {
            ChannelEvent::ReceivedEvent(_, RemotePeer(remote_udp_peer)) => {
                self.on_remote_peer(remote_udp_peer)
            }
            ChannelEvent::ReceivedEvent(receive_meta_data, SendCompletedStep(state_message)) => {
                self.on_completed_step(receive_meta_data, state_message)
            }
            ChannelEvent::ReceivedEvent(receive_meta_data, SendTimeMessage(time_message)) => {
                self.on_time_message(receive_meta_data, time_message)
            }
            ChannelEvent::ReceivedEvent(receive_meta_data, SendInputMessage(input_message)) => {
                self.on_input_message(receive_meta_data, input_message)
            }
            ChannelEvent::ReceivedEvent(
                receive_meta_data,
                SendServerInputMessage(server_input_message),
            ) => self.on_server_input_message(receive_meta_data, server_input_message),
            ChannelEvent::Timeout => EventHandleResult::WaitForNextEvent,
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent,
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        ()
    }
}
