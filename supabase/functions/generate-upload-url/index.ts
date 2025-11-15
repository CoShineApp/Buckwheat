import { createClient } from '@supabase/supabase-js'
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3'
import { getSignedUrl } from '@aws-sdk/s3-request-presigner'

Deno.serve(async (req) => {
  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    return new Response(null, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'POST, OPTIONS',
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

    // Check quota
    const { data: profile } = await supabase
      .from('profiles')
      .select('storage_used, storage_limit')
      .eq('id', user.id)
      .single()

    if (!profile) {
      return new Response(JSON.stringify({ error: 'Profile not found' }), {
        status: 404,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    const { fileName, fileSize } = await req.json()
    
    if (profile.storage_used + fileSize > profile.storage_limit) {
      return new Response(JSON.stringify({ error: 'Quota exceeded' }), {
        status: 413,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      })
    }

    // Generate signed upload URL
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

    const key = `${user.id}/${crypto.randomUUID()}/${fileName}`
    const command = new PutObjectCommand({
      Bucket: Deno.env.get('B2_BUCKET_NAME_UPLOADS')!,
      Key: key,
    })

    const uploadUrl = await getSignedUrl(s3Client, command, { expiresIn: 900 }) // 15 min

    // Create database record with UPLOADING status
    const { data: upload, error: insertError } = await supabase
      .from('uploads')
      .insert({
        user_id: user.id,
        filename: fileName,
        b2_file_name: key,
        file_size: fileSize,
        status: 'UPLOADING',
      })
      .select()
      .single()

    if (insertError) {
      console.error('Failed to create upload record:', insertError)
      throw insertError
    }

    // Update storage usage
    const newUsage = profile.storage_used + fileSize
    const { error: updateError } = await supabase
      .from('profiles')
      .update({ storage_used: newUsage })
      .eq('id', user.id)

    if (updateError) {
      console.error('Failed to update storage usage:', updateError)
      // Don't throw - upload record is created, just log the error
    }

    console.log(`✅ Upload record created: ${key}, new usage: ${(newUsage / 1024 / 1024).toFixed(2)} MB`)

    return new Response(JSON.stringify({ uploadUrl, upload }), {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  } catch (error) {
    console.error('Error in generate-upload-url:', error)
    return new Response(JSON.stringify({ error: error.message }), {
      status: 500,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  }
})

