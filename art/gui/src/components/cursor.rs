/// A cursor represents a position in a range or slice.
///
/// The cursor can be moved forward and backward, but can never exceed the bounds of the range.
pub struct Cursor {
    length: usize,
    position: usize,
}

impl Cursor {
    /// Creates a new instance.
    ///
    /// # Arguments
    ///
    /// * `length`: The number of positions.
    pub fn new(length: usize) -> Self {
        assert_ne!(length, 0);
        Self {
            length,
            position: 0,
        }
    }

    /// Retrieves the current position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Resets the cursor to the initial position.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Moves the cursor backward at most the provided number of steps.
    ///
    /// # Parameters
    /// * `count`: The maximum number of steps to move the cursor.
    ///
    /// # Returns
    /// The number of steps that the cursor was actually moved.
    #[allow(unused)]
    pub fn move_backward(&mut self, count: usize) -> usize {
        let distance = self.position.min(count);
        self.position -= distance;
        distance
    }

    /// Moves the cursor forward at most the provided number of steps.
    ///
    /// # Parameters
    /// * `count`: The maximum number of steps to move the cursor.
    ///
    /// # Returns
    /// The number of steps that the cursor was actually moved.
    pub fn move_forward(&mut self, count: usize) -> usize {
        let distance = ((self.length - 1) - self.position).min(count);
        self.position += distance;
        distance
    }

    /// Moves the cursor forward one step.
    ///
    /// # Returns
    /// The new position of the cursor or `None` if the cursor is at the upper bound.
    pub fn next(&mut self) -> Option<usize> {
        if self.move_forward(1) == 0 {
            None
        } else {
            Some(self.position)
        }
    }
}
