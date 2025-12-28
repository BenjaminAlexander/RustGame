use {
    crate::{
        Play,
        client::udp_output::UdpOutput,
        frame_manager::{
            ClientProvidedInput,
            FrameIndexAndState,
            InputCoordinates,
            ObserveFrames,
        },
        messaging::ToServerInputMessage,
        state_channel::StateSender,
    },
    log::warn,
    std::ops::ControlFlow,
};

pub struct ClientFrameObserver<Game: Play> {
    state_sender: StateSender<Game>,
    udp_output: UdpOutput<Game>,
}

impl<Game: Play> ClientFrameObserver<Game> {
    pub fn new(state_sender: StateSender<Game>, udp_output: UdpOutput<Game>) -> Self {
        Self {
            state_sender,
            udp_output,
        }
    }
}

impl<Game: Play> ObserveFrames for ClientFrameObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = false;

    fn new_state(
        &self,
        _is_state_authoritative: bool,
        step_message: FrameIndexAndState<Game>,
    ) -> ControlFlow<()> {
        let result = self.state_sender.send_state(step_message);

        if result.is_err() {
            warn!("Failed to send new state to render receiver");
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn input_authoritatively_missing(&self, _: InputCoordinates) -> ControlFlow<()> {
        panic!("The client should never declare inputs authoritatively missing");
    }

    fn input_authoritative(
        &self,
        input_coordinates: InputCoordinates,
        input: <Self::Game as Play>::InputAggregate,
    ) -> ControlFlow<()> {
        let message = ToServerInputMessage::<Game>::new(
            input_coordinates,
            ClientProvidedInput::Authoritative(input),
        );

        let send_result = self.udp_output.send_input(message);

        if send_result.is_err() {
            warn!("Failed to send InputMessage to Udp Output");
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    }
}
