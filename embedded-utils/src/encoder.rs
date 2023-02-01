use embedded_hal::digital::v2::InputPin;
use core::fmt::Debug;

pub struct Encoder<PinA, PinB, E>
where
    PinA: InputPin<Error = E>,
    PinB: InputPin<Error = E>,
    E: Debug
{
    a: PinA,
    b: PinB,
    state: u8,
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Positive,
    Negative,
    None,
    Skip
}

impl<PinA, PinB, E> Encoder<PinA, PinB, E>
where
    PinA: InputPin<Error = E>,
    PinB: InputPin<Error = E>,
    E: Debug
{
    pub fn new(a: PinA, b: PinB) -> Encoder<PinA, PinB ,E> {
        Encoder { a, b, state: 0 }
    }

    // Decode quadrature state
    //     a 0 1 1 0
    //     b 0 0 1 1
    // state 0 1 2 3
    fn state(a: bool, b: bool) -> u8 {
        return if a==b {0} else {1} + if b {2} else {0}
    }

    pub fn update(&mut self) -> Result<Direction, E> {

        let old_state = self.state;
        self.state = Self::state(self.a.is_high()?, self.b.is_high()?);

        match (4 + self.state - old_state) & 3 {
            0 => Ok(Direction::None),
            1 => Ok(Direction::Positive),
            3 => Ok(Direction::Negative),
            _ => Ok(Direction::Skip)
        }
    }
    pub fn destroy(self) -> (PinA, PinB)
    {
        (self.a, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::pin::{Mock as PinMock, Transaction as PinTransaction, State as PinState};

    #[test]
    fn one_positive_rev_from_state_0() {
        let a = PinMock::new(&[
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low)
        ]);
        let b = PinMock::new(&[
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::Low)
        ]);

        let mut enc = Encoder::new(a, b);
        assert_eq!(enc.update(), Ok(Direction::None));
        assert_eq!(enc.update(), Ok(Direction::Positive));
        assert_eq!(enc.update(), Ok(Direction::Positive));
        assert_eq!(enc.update(), Ok(Direction::Positive));
        assert_eq!(enc.update(), Ok(Direction::Positive));
        let (mut a,mut b) = enc.destroy();

        a.done();
        b.done();
    }

    #[test]
    fn one_negative_rev_from_2() {
        let a = PinMock::new(&[
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::High)
        ]);
        let b = PinMock::new(&[
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::High),
            PinTransaction::get(PinState::High)
        ]);

        let mut enc = Encoder::new(a, b);
        assert_eq!(enc.update(), Ok(Direction::Skip));
        assert_eq!(enc.update(), Ok(Direction::Negative));
        assert_eq!(enc.update(), Ok(Direction::Negative));
        assert_eq!(enc.update(), Ok(Direction::Negative));
        assert_eq!(enc.update(), Ok(Direction::Negative));
        let (mut a,mut b) = enc.destroy();

        a.done();
        b.done();
    }
}