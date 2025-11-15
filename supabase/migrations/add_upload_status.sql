-- Add status column to uploads table
ALTER TABLE uploads ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'UPLOADING';

-- Create index for faster queries on status
CREATE INDEX IF NOT EXISTS idx_uploads_status ON uploads(status);

-- Add check constraint to ensure valid status values
ALTER TABLE uploads DROP CONSTRAINT IF EXISTS uploads_status_check;
ALTER TABLE uploads ADD CONSTRAINT uploads_status_check CHECK (status IN ('UPLOADING', 'UPLOADED', 'FAILED'));

-- Update existing records to 'UPLOADED' (assume they're already uploaded)
UPDATE uploads SET status = 'UPLOADED' WHERE status = 'UPLOADING';

