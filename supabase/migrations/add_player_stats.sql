-- Player game statistics table
-- Tracks per-game, per-player stats for Melee matches

CREATE TABLE IF NOT EXISTS player_game_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    
    -- Game metadata
    game_date TIMESTAMPTZ NOT NULL,
    slp_file_hash TEXT,  -- Hash of .slp file for deduplication
    stage_id SMALLINT NOT NULL,
    game_duration_frames INTEGER NOT NULL,
    
    -- Player info
    player_port SMALLINT NOT NULL CHECK (player_port BETWEEN 1 AND 4),
    player_tag TEXT NOT NULL,
    character_id SMALLINT NOT NULL,
    opponent_character_id SMALLINT,
    
    -- L-Cancel stats
    l_cancel_hit INTEGER NOT NULL DEFAULT 0,
    l_cancel_missed INTEGER NOT NULL DEFAULT 0,
    
    -- Neutral & opening stats
    neutral_wins INTEGER NOT NULL DEFAULT 0,
    neutral_losses INTEGER NOT NULL DEFAULT 0,
    openings INTEGER NOT NULL DEFAULT 0,
    damage_per_opening REAL,
    openings_per_kill REAL,
    
    -- Kill stats
    kills INTEGER NOT NULL DEFAULT 0,
    deaths INTEGER NOT NULL DEFAULT 0,
    avg_kill_percent REAL,
    total_damage_dealt REAL NOT NULL DEFAULT 0,
    total_damage_taken REAL NOT NULL DEFAULT 0,
    
    -- Tech skill stats
    successful_techs INTEGER NOT NULL DEFAULT 0,
    missed_techs INTEGER NOT NULL DEFAULT 0,
    wavedash_count INTEGER NOT NULL DEFAULT 0,
    dashdance_count INTEGER NOT NULL DEFAULT 0,
    
    -- Input stats
    apm REAL NOT NULL DEFAULT 0,
    grab_attempts INTEGER NOT NULL DEFAULT 0,
    grab_success INTEGER NOT NULL DEFAULT 0,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_player_game_stats_user_id ON player_game_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_player_game_stats_player_tag ON player_game_stats(player_tag);
CREATE INDEX IF NOT EXISTS idx_player_game_stats_character ON player_game_stats(character_id);
CREATE INDEX IF NOT EXISTS idx_player_game_stats_game_date ON player_game_stats(game_date DESC);
CREATE INDEX IF NOT EXISTS idx_player_game_stats_slp_hash ON player_game_stats(slp_file_hash);

-- RLS policies
ALTER TABLE player_game_stats ENABLE ROW LEVEL SECURITY;

-- Users can view their own stats
CREATE POLICY "Users can view their own stats"
    ON player_game_stats FOR SELECT
    USING (auth.uid() = user_id);

-- Users can insert their own stats
CREATE POLICY "Users can insert their own stats"
    ON player_game_stats FOR INSERT
    WITH CHECK (auth.uid() = user_id);

-- Users can update their own stats
CREATE POLICY "Users can update their own stats"
    ON player_game_stats FOR UPDATE
    USING (auth.uid() = user_id);

-- Users can delete their own stats
CREATE POLICY "Users can delete their own stats"
    ON player_game_stats FOR DELETE
    USING (auth.uid() = user_id);

-- Create updated_at trigger function if it doesn't exist
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger for updated_at
DROP TRIGGER IF EXISTS update_player_game_stats_updated_at ON player_game_stats;
CREATE TRIGGER update_player_game_stats_updated_at
    BEFORE UPDATE ON player_game_stats
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

