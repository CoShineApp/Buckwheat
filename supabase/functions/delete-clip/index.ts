import { createClient } from '@supabase/supabase-js'
import { S3Client, DeleteObjectCommand } from '@aws-sdk/client-s3'

Deno.serve(async (req) => {
  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    return new Response(null, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'POST, DELETE, OPTIONS',
        'Access-Control-Allow-Headers': 'authorization, content-type, x-client-info, apikey',
      },
    })
  }

  try {
    const supabase = createClient(
      Deno.env.get('SUPABASE_URL')!,
      Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!
    )

    // Get auth header (optional for clips - can be device-based)
    const authHeader = req.headers.get('Authorization')
    let userId = null
    
    if (authHeader) {
      const { data: { user } } = await supabase.auth.getUser(
        authHeader.replace('Bearer ', '')
      )
      userId = user?.id
    }

    const { clipId, deviceId } = await req.json()

    if (!clipId) {
      return new Response(JSON.stringify({ error: 'Missing clipId' }), {
        status: 400,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    // Get the clip record - verify ownership by user_id or device_id
    let query = supabase
      .from('clips')
      .select('*')
      .eq('id', clipId)

    const { data: clip, error: fetchError } = await query.single()

    if (fetchError || !clip) {
      return new Response(JSON.stringify({ error: 'Clip not found' }), {
        status: 404,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    // Verify ownership - must match user_id or device_id
    const isOwner = (userId && clip.user_id === userId) || 
                    (deviceId && clip.device_id === deviceId)
    
    if (!isOwner) {
      return new Response(JSON.stringify({ error: 'Unauthorized - not the clip owner' }), {
        status: 403,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    // Delete from B2/R2 clips bucket
    const b2Endpoint = Deno.env.get('B2_ENDPOINT')!
    const endpoint = b2Endpoint.startsWith('http') ? b2Endpoint : `https://${b2Endpoint}`
    
    const s3Client = new S3Client({
      region: Deno.env.get('B2_REGION')!,
      endpoint: endpoint,
      credentials: {
        accessKeyId: Deno.env.get('B2_KEY_ID')!,
        secretAccessKey: Deno.env.get('B2_APPLICATION_KEY')!,
      },
    })

    // Delete the file from storage
    if (clip.b2_file_name) {
      try {
        await s3Client.send(new DeleteObjectCommand({
          Bucket: Deno.env.get('B2_BUCKET_NAME_CLIPS')!,
          Key: clip.b2_file_name,
        }))
        console.log(`✅ Deleted clip from B2: ${clip.b2_file_name}`)
      } catch (deleteError) {
        console.error('Failed to delete clip from B2 (continuing anyway):', deleteError)
        // Continue anyway - we still want to remove the DB record
      }
    }

    // Delete the database record
    const { error: deleteError } = await supabase
      .from('clips')
      .delete()
      .eq('id', clipId)

    if (deleteError) {
      throw deleteError
    }

    // If user is authenticated, update their storage usage
    if (userId) {
      const { data: profile } = await supabase
        .from('profiles')
        .select('storage_used')
        .eq('id', userId)
        .single()

      if (profile) {
        const newUsage = Math.max(0, (profile.storage_used || 0) - (clip.file_size || 0))
        const { error: updateError } = await supabase
          .from('profiles')
          .update({ storage_used: newUsage })
          .eq('id', userId)

        if (updateError) {
          console.error('Failed to update storage usage:', updateError)
        } else {
          console.log(`✅ Updated storage: ${(profile.storage_used / 1024 / 1024).toFixed(2)} MB → ${(newUsage / 1024 / 1024).toFixed(2)} MB`)
        }
      }
    }

    console.log(`✅ Deleted clip: ${clipId} (share_code: ${clip.share_code})`)

    return new Response(JSON.stringify({ success: true }), {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  } catch (error) {
    console.error('Error in delete-clip:', error)
    return new Response(JSON.stringify({ error: error.message }), {
      status: 500,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  }
})
