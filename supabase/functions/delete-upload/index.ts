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

    // Verify user from auth token
    const authHeader = req.headers.get('Authorization')
    if (!authHeader) {
      return new Response(JSON.stringify({ error: 'Missing authorization header' }), {
        status: 401,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    const { data: { user }, error: authError } = await supabase.auth.getUser(
      authHeader.replace('Bearer ', '')
    )
    
    if (authError || !user) {
      return new Response(JSON.stringify({ error: 'Unauthorized' }), {
        status: 401,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    const { uploadId } = await req.json()

    if (!uploadId) {
      return new Response(JSON.stringify({ error: 'Missing uploadId' }), {
        status: 400,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    // Get the upload record to verify ownership and get file info
    const { data: upload, error: fetchError } = await supabase
      .from('uploads')
      .select('*')
      .eq('id', uploadId)
      .eq('user_id', user.id)
      .single()

    if (fetchError || !upload) {
      return new Response(JSON.stringify({ error: 'Upload not found or unauthorized' }), {
        status: 404,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    // Delete from B2/R2 storage
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
    if (upload.b2_file_name) {
      try {
        await s3Client.send(new DeleteObjectCommand({
          Bucket: Deno.env.get('B2_BUCKET_NAME_UPLOADS')!,
          Key: upload.b2_file_name,
        }))
        console.log(`✅ Deleted file from B2: ${upload.b2_file_name}`)
      } catch (deleteError) {
        console.error('Failed to delete from B2 (continuing anyway):', deleteError)
        // Continue anyway - we still want to remove the DB record
      }
    }

    // Delete the database record
    const { error: deleteError } = await supabase
      .from('uploads')
      .delete()
      .eq('id', uploadId)

    if (deleteError) {
      throw deleteError
    }

    // Update storage usage - subtract the file size
    const { data: profile } = await supabase
      .from('profiles')
      .select('storage_used')
      .eq('id', user.id)
      .single()

    if (profile) {
      const newUsage = Math.max(0, (profile.storage_used || 0) - (upload.file_size || 0))
      const { error: updateError } = await supabase
        .from('profiles')
        .update({ storage_used: newUsage })
        .eq('id', user.id)

      if (updateError) {
        console.error('Failed to update storage usage:', updateError)
      } else {
        console.log(`✅ Updated storage: ${(profile.storage_used / 1024 / 1024).toFixed(2)} MB → ${(newUsage / 1024 / 1024).toFixed(2)} MB`)
      }
    }

    console.log(`✅ Deleted upload: ${uploadId}`)

    return new Response(JSON.stringify({ success: true }), {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  } catch (error) {
    console.error('Error in delete-upload:', error)
    return new Response(JSON.stringify({ error: error.message }), {
      status: 500,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  }
})
