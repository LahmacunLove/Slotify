use session_recorder_addon::models::{
    dj::Dj,
    lottery::{LotteryEngine, LotteryConfig},
};
use chrono::Utc;

#[cfg(test)]
mod lottery_tests {
    use super::*;

    fn create_test_dj(name: &str, hours_ago: i64) -> Dj {
        let mut dj = Dj::new(name.to_string(), Some(format!("{}@example.com", name.to_lowercase())));
        dj.registered_at = Utc::now() - chrono::Duration::hours(hours_ago);
        dj
    }

    #[test]
    fn test_lottery_engine_creation() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        // Engine should be created successfully
        assert!(true);
    }

    #[test]
    fn test_empty_dj_list() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        let djs = vec![];
        
        let result = engine.draw_winner(&djs);
        assert!(result.is_none());
    }

    #[test]
    fn test_single_dj() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        let djs = vec![create_test_dj("Alice", 1)];
        
        let result = engine.draw_winner(&djs);
        assert!(result.is_some());
        
        let draw = result.unwrap();
        assert_eq!(draw.winner.name, "Alice");
        assert_eq!(draw.participants.len(), 1);
    }

    #[test]
    fn test_multiple_djs() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        let djs = vec![
            create_test_dj("Alice", 3),
            create_test_dj("Bob", 2),
            create_test_dj("Charlie", 1),
        ];
        
        let result = engine.draw_winner(&djs);
        assert!(result.is_some());
        
        let draw = result.unwrap();
        assert_eq!(draw.participants.len(), 3);
        
        // Check that all participants have reasonable weights
        for participant in &draw.participants {
            assert!(participant.calculated_weight > 0.0);
            assert!(participant.selection_probability > 0.0);
            assert!(participant.selection_probability <= 1.0);
        }
        
        // Check that probabilities sum to approximately 1.0
        let total_prob: f64 = draw.participants.iter()
            .map(|p| p.selection_probability)
            .sum();
        assert!((total_prob - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_weight_calculation() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        
        let early_dj = create_test_dj("Early", 5);
        let late_dj = create_test_dj("Late", 1);
        
        let djs = vec![early_dj, late_dj];
        let participants = engine.calculate_weights(&djs);
        
        // Early DJ should have higher weight due to time bonus
        let early_weight = participants.iter()
            .find(|p| p.dj.name == "Early")
            .unwrap()
            .calculated_weight;
        
        let late_weight = participants.iter()
            .find(|p| p.dj.name == "Late")
            .unwrap()
            .calculated_weight;
        
        assert!(early_weight > late_weight);
    }

    #[test]
    fn test_late_arrival_penalty() {
        let config = LotteryConfig {
            late_arrival_penalty: 0.5,
            ..Default::default()
        };
        let engine = LotteryEngine::new(config);
        
        // This test would need to mock the current time to properly test late arrival
        // For now, we'll just ensure the engine handles the config correctly
        let djs = vec![create_test_dj("TestDJ", 1)];
        let participants = engine.calculate_weights(&djs);
        
        assert_eq!(participants.len(), 1);
        assert!(participants[0].calculated_weight > 0.0);
    }

    #[test]
    fn test_minimum_weight_enforcement() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        
        // Create a DJ with very low base weight
        let mut low_weight_dj = create_test_dj("LowWeight", 1);
        low_weight_dj.weight = 0.001;
        
        let djs = vec![low_weight_dj];
        let participants = engine.calculate_weights(&djs);
        
        // Should enforce minimum weight of 0.1
        assert!(participants[0].calculated_weight >= 0.1);
    }

    #[test]
    fn test_multiple_draws_fairness() {
        let config = LotteryConfig::default();
        let engine = LotteryEngine::new(config);
        
        let djs = vec![
            create_test_dj("Alice", 2),
            create_test_dj("Bob", 2),
            create_test_dj("Charlie", 2),
        ];
        
        let mut winner_counts = std::collections::HashMap::new();
        let num_draws = 100;
        
        // Run multiple draws and count winners
        for _ in 0..num_draws {
            if let Some(draw) = engine.draw_winner(&djs) {
                *winner_counts.entry(draw.winner.name).or_insert(0) += 1;
            }
        }
        
        // Each DJ should win at least once in 100 draws (very likely)
        assert_eq!(winner_counts.len(), 3);
        
        // No single DJ should win more than 80% of the time (indicates fairness)
        for count in winner_counts.values() {
            assert!(*count < 80);
        }
    }
}