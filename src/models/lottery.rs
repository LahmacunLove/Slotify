use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};
use crate::models::dj::{Dj, DjResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct LotteryDraw {
    pub winner: DjResponse,
    pub participants: Vec<LotteryParticipant>,
    pub drawn_at: DateTime<Utc>,
    pub algorithm_used: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotteryParticipant {
    pub dj: DjResponse,
    pub calculated_weight: f64,
    pub selection_probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotteryConfig {
    pub base_weight: f64,
    pub late_arrival_penalty: f64,
    pub time_block_hours: u32,
    pub enable_time_blocking: bool,
}

impl Default for LotteryConfig {
    fn default() -> Self {
        Self {
            base_weight: 1.0,
            late_arrival_penalty: 0.5,
            time_block_hours: 2,
            enable_time_blocking: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotteryStatistics {
    pub total_draws: usize,
    pub unique_winners: usize,
    pub average_weight: f64,
    pub fairness_score: f64, // 0-1, where 1 is perfectly fair
}

pub struct LotteryEngine {
    config: LotteryConfig,
}

impl LotteryEngine {
    pub fn new(config: LotteryConfig) -> Self {
        Self { config }
    }

    pub fn calculate_weights(&self, djs: &[Dj]) -> Vec<LotteryParticipant> {
        let total_participants = djs.len() as f64;
        let mut participants = Vec::new();

        for dj in djs {
            let calculated_weight = self.calculate_individual_weight(dj);
            participants.push(LotteryParticipant {
                dj: dj.clone().into(),
                calculated_weight,
                selection_probability: 0.0, // Will be calculated after all weights are known
            });
        }

        // Calculate probabilities
        let total_weight: f64 = participants.iter().map(|p| p.calculated_weight).sum();
        for participant in &mut participants {
            participant.selection_probability = participant.calculated_weight / total_weight;
        }

        participants
    }

    fn calculate_individual_weight(&self, dj: &Dj) -> f64 {
        let mut weight = dj.weight * self.config.base_weight;

        // Apply late arrival penalty
        let current_hour = Utc::now().hour();
        if current_hour >= 24 {
            weight *= self.config.late_arrival_penalty;
        }

        // Apply time-based fairness (earlier arrivals get slightly higher weight)
        let hours_registered = Utc::now()
            .signed_duration_since(dj.registered_at)
            .num_hours() as f64;
        
        if hours_registered > 0.0 {
            // Small bonus for being registered longer (max 20% bonus)
            let time_bonus = (hours_registered / 10.0).min(0.2);
            weight *= 1.0 + time_bonus;
        }

        weight.max(0.1) // Ensure minimum weight
    }

    pub fn draw_winner(&self, djs: &[Dj]) -> Option<LotteryDraw> {
        if djs.is_empty() {
            return None;
        }

        let participants = self.calculate_weights(djs);
        let total_weight: f64 = participants.iter().map(|p| p.calculated_weight).sum();
        
        if total_weight <= 0.0 {
            return None;
        }

        // Use rand to generate random number
        let random_value = rand::random::<f64>() * total_weight;
        let mut cumulative_weight = 0.0;

        for participant in &participants {
            cumulative_weight += participant.calculated_weight;
            if random_value <= cumulative_weight {
                return Some(LotteryDraw {
                    winner: participant.dj.clone(),
                    participants,
                    drawn_at: Utc::now(),
                    algorithm_used: "weighted_random".to_string(),
                });
            }
        }

        // Fallback: return last participant (should never happen)
        if let Some(last) = participants.last() {
            Some(LotteryDraw {
                winner: last.dj.clone(),
                participants,
                drawn_at: Utc::now(),
                algorithm_used: "weighted_random_fallback".to_string(),
            })
        } else {
            None
        }
    }
}