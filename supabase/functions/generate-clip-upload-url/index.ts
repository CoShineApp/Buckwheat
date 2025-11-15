import { createClient } from '@supabase/supabase-js'
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3'
import { getSignedUrl } from '@aws-sdk/s3-request-presigner'

function generateShareCode(): string {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'
  return Array.from({ length: 8 }, () => 
    chars[Math.floor(Math.random() * chars.length)]
  ).join('')
}

Deno.serve(async (req) => {
  console.log('📥 generate-clip-upload-url called, method:', req.method)
  
  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    console.log('✅ CORS preflight request')
    return new Response(null, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'POST, OPTIONS',
        'Access-Control-Allow-Headers': 'authorization, content-type, x-client-info, apikey',
      },
    })
  }

  try {
    console.log('🔧 Step 1: Creating Supabase client')
    const supabaseUrl = Deno.env.get('SUPABASE_URL')
    const supabaseServiceKey = Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')
    
    if (!supabaseUrl || !supabaseServiceKey) {
      throw new Error(`Missing env vars: SUPABASE_URL=${!!supabaseUrl}, SUPABASE_SERVICE_ROLE_KEY=${!!supabaseServiceKey}`)
    }
    
    const supabase = createClient(supabaseUrl, supabaseServiceKey)
    console.log('✅ Supabase client created')

    // Optional: verify user if authenticated
    console.log('🔧 Step 2: Checking auth')
    const authHeader = req.headers.get('Authorization')
    let userId = null
    if (authHeader) {
      console.log('🔐 Auth header present, verifying user')
      const { data: { user }, error: authError } = await supabase.auth.getUser(authHeader.replace('Bearer ', ''))
      if (authError) {
        console.warn('⚠️ Auth error (non-fatal):', authError)
      } else {
        userId = user?.id
        console.log('✅ User verified:', userId)
      }
    } else {
      console.log('ℹ️ No auth header, proceeding as anonymous')
    }

    console.log('🔧 Step 3: Parsing request body')
    const body = await req.json()
    console.log('📦 Request body:', { 
      fileName: body.fileName, 
      fileSize: body.fileSize, 
      deviceId: body.deviceId,
      hasMetadata: !!body.metadata 
    })
    
    const { fileName, fileSize, deviceId, metadata } = body
    
    if (!fileName || !fileSize || !deviceId) {
      throw new Error(`Missing required fields: fileName=${!!fileName}, fileSize=${!!fileSize}, deviceId=${!!deviceId}`)
    }
    
    // Generate share code
    console.log('🔧 Step 4: Generating share code')
    const shareCode = generateShareCode()
    console.log('✅ Share code generated:', shareCode)
    
    // Generate signed upload URL for clips bucket
    console.log('🔧 Step 5: Setting up S3 client')
    const b2Endpoint = Deno.env.get('B2_ENDPOINT')
    const b2Region = Deno.env.get('B2_REGION')
    const b2KeyId = Deno.env.get('B2_KEY_ID')
    const b2AppKey = Deno.env.get('B2_APPLICATION_KEY')
    const b2BucketClips = Deno.env.get('B2_BUCKET_NAME_CLIPS')
    
    console.log('🔍 Env vars check:', {
      B2_ENDPOINT: !!b2Endpoint,
      B2_REGION: !!b2Region,
      B2_KEY_ID: !!b2KeyId,
      B2_APPLICATION_KEY: !!b2AppKey,
      B2_BUCKET_NAME_CLIPS: !!b2BucketClips
    })
    
    if (!b2Endpoint || !b2Region || !b2KeyId || !b2AppKey || !b2BucketClips) {
      throw new Error('Missing B2 environment variables')
    }
    
    const endpoint = b2Endpoint.startsWith('http') ? b2Endpoint : `https://${b2Endpoint}`
    console.log('✅ B2 endpoint:', endpoint)
    
    const s3Client = new S3Client({
      region: b2Region,
      endpoint: endpoint,
      credentials: {
        accessKeyId: b2KeyId,
        secretAccessKey: b2AppKey,
      },
    })
    console.log('✅ S3 client created')

    const key = `clips/${shareCode}/${fileName}`
    console.log('🔧 Step 6: Creating PutObjectCommand, key:', key)
    const command = new PutObjectCommand({
      Bucket: b2BucketClips,
      Key: key,
      ContentType: 'video/mp4',
    })
    console.log('✅ PutObjectCommand created')

    console.log('🔧 Step 7: Generating signed URL')
    const uploadUrl = await getSignedUrl(s3Client, command, { expiresIn: 900 }) // 15 min
    console.log('✅ Signed URL generated:', uploadUrl.substring(0, 50) + '...')

    // Create database record (will be updated to complete after upload)
    console.log('🔧 Step 8: Creating database record')
    const insertData = {
      user_id: userId,
      device_id: deviceId,
      filename: fileName,
      b2_file_name: key,
      file_size: fileSize,
      share_code: shareCode,
      metadata: metadata || null,
    }
    console.log('📝 Insert data:', { ...insertData, metadata: metadata ? 'present' : 'null' })
    
    const { data: clip, error: insertError } = await supabase
      .from('clips')
      .insert(insertData)
      .select()
      .single()

    if (insertError) {
      console.error('❌ Failed to create clip record:', insertError)
      throw insertError
    }
    console.log('✅ Clip record created:', clip.id)

    console.log('✅ Success! Returning response')
    return new Response(JSON.stringify({ uploadUrl, clip, shareCode }), {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  } catch (error) {
    console.error('❌ Error in generate-clip-upload-url:', error)
    console.error('❌ Error details:', {
      message: error.message,
      stack: error.stack,
      name: error.name
    })
    return new Response(JSON.stringify({ 
      error: error.message,
      details: error.stack 
    }), {
      status: 500,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      }
    })
  }
})

