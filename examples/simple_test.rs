// Simple test example to verify basic functionality

use session_recorder_addon::models::{
    dj::Dj,
    lottery::{LotteryEngine, LotteryConfig},
};

fn main() {
    println!("🎵 DJ Session Recorder - Simple Test");
    println!("=====================================");

    // Test 1: Create a DJ
    println!("\n1. Testing DJ Creation:");
    let dj = Dj::new("Test DJ".to_string(), Some("test@example.com".to_string()));
    println!("   ✅ Created DJ: {} (ID: {})", dj.name, dj.id);
    println!("   📧 Email: {:?}", dj.email);
    println!("   ⚖️ Weight: {}", dj.weight);

    // Test 2: Create lottery engine
    println!("\n2. Testing Lottery Engine:");
    let config = LotteryConfig::default();
    let engine = LotteryEngine::new(config);
    println!("   ✅ Lottery engine created");

    // Test 3: Test with multiple DJs
    println!("\n3. Testing Lottery with Multiple DJs:");
    let djs = vec![
        Dj::new("DJ Alpha".to_string(), Some("alpha@example.com".to_string())),
        Dj::new("DJ Beta".to_string(), Some("beta@example.com".to_string())),
        Dj::new("DJ Gamma".to_string(), None),
    ];

    println!("   📋 Created {} DJs for lottery", djs.len());
    for (i, dj) in djs.iter().enumerate() {
        println!("      {}. {} (Weight: {})", i + 1, dj.name, dj.weight);
    }

    // Test 4: Calculate weights
    println!("\n4. Testing Weight Calculation:");
    let participants = engine.calculate_weights(&djs);
    for participant in &participants {
        println!("   {} - Weight: {:.2}, Probability: {:.1}%", 
            participant.dj.name, 
            participant.calculated_weight,
            participant.selection_probability * 100.0
        );
    }

    // Test 5: Run lottery
    println!("\n5. Testing Lottery Draw:");
    if let Some(draw) = engine.draw_winner(&djs) {
        println!("   🎉 Winner: {}", draw.winner.name);
        println!("   🎲 Algorithm: {}", draw.algorithm_used);
        println!("   📊 Total participants: {}", draw.participants.len());
    } else {
        println!("   ❌ No winner drawn");
    }

    // Test 6: Multiple draws to test fairness
    println!("\n6. Testing Fairness (10 draws):");
    let mut winners = std::collections::HashMap::new();
    for i in 1..=10 {
        if let Some(draw) = engine.draw_winner(&djs) {
            *winners.entry(draw.winner.name.clone()).or_insert(0) += 1;
        }
    }

    for (name, count) in winners {
        println!("   {} won {} times", name, count);
    }

    println!("\n✅ All tests completed successfully!");
    println!("🚀 You can now test the full system with 'cargo run --bin server'");
}