use crate::track::strip_unnessecary;
use std::collections::HashMap;
use std::io::Read;
use ndarray::{Array1, Array2, ArrayBase};
use ndarray_npy::read_npy;

pub struct AutoEncoder {
    encoder_weights: Array2<f32>,
    encoder_biases: Array1<f32>,
    decoder_weights: Array2<f32>,
    decoder_biases: Array1<f32>,
    genre_index: Vec<String>
}

impl AutoEncoder {
    pub fn new() -> anyhow::Result<Self> {
        let model_path = "./models/";

        let encoder_weights = read_npy("./models/encoder_weights.npy")?;
        let encoder_biases = read_npy("./models/encoder_biases.npy")?;
        let decoder_weights = read_npy("./models/decoder_weights.npy")?;
        let decoder_biases = read_npy("./models/decoder_biases.npy")?;

        let genre_index = std::fs::read_to_string("./models/genrelist")?.split("\n").map(|genre| genre.to_string()).collect();

        Ok(AutoEncoder {
            genre_index,
            encoder_weights,
            encoder_biases,
            decoder_weights,
            decoder_biases,
        })
    }

    pub fn encode(&self, input: Array1<f32>) -> anyhow::Result<Array1<f32>> {
        assert_eq!(self.encoder_weights.shape()[0], input.len());
        assert_eq!(self.encoder_biases.len(), self.encoder_weights.shape()[1]);
        let mut result = input.dot(&self.encoder_weights) + self.encoder_biases.clone();
        
        // ReLu Activation
        result = result.clamp(0.0, 1_000_000.0);

        Ok(result)
    }

    pub fn decode(&self, input: Array1<f32>) -> anyhow::Result<Array1<f32>> {
        assert_eq!(self.decoder_weights.shape()[0], input.len());
        assert_eq!(self.decoder_biases.len(), self.decoder_weights.shape()[1]);
        let mut result = input.dot(&self.decoder_weights) + self.decoder_biases.clone();

        // Sigmoid Activation
        result = 1.0 / (1.0 + (result * -1.0).exp());

        Ok(result)
    }

    // Turns genres into a one-hot encoding before they are encoded further
    pub fn genres_to_vec(&self, genres: Vec<String>) -> Array1<f32> {
        let mut encoding = ArrayBase::from_vec(vec![0.0; self.genre_index.len()]);

        for genre in genres {
            if let Some(idx) = self
                .genre_index
                .iter()
                .position(|other_genre| *other_genre == strip_unnessecary(&genre))
            {
                encoding[idx] = 1.0;
            }
        }

        return encoding;
    }
}
