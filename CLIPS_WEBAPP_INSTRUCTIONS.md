# Clips.peppi.app - Web App Instructions

## Overview
Create a simple, clean web app for viewing public clips shared from the Peppi desktop app. The app should be deployed to Vercel at `clips.peppi.app`.

## Tech Stack
- **Framework**: SvelteKit (latest)
- **Hosting**: Vercel
- **Styling**: TailwindCSS
- **Video Player**: Native HTML5 video or Plyr.js
- **API**: Supabase (read-only access to clips table)

## Project Structure

```
clips-webapp/
├── src/
│   ├── routes/
│   │   ├── +page.svelte          # Home page (optional - can redirect)
│   │   └── [shareCode]/
│   │       └── +page.svelte      # Clip viewer page
│   ├── lib/
│   │   ├── supabase.ts           # Supabase client setup
│   │   └── utils.ts              # Helper functions
│   └── app.html
├── static/
│   └── favicon.ico
├── package.json
├── svelte.config.js
├── tailwind.config.js
├── vite.config.js
└── vercel.json                    # Vercel deployment config
```

## Environment Variables

Create a `.env` file (and set in Vercel dashboard):

```env
PUBLIC_SUPABASE_URL=your_supabase_url
PUBLIC_SUPABASE_ANON_KEY=your_supabase_anon_key
PUBLIC_CLIPS_CDN_URL=https://clips.peppi.app
```

## Key Features

### 1. Route Structure
- `clips.peppi.app/` → Home/landing page (optional)
- `clips.peppi.app/[shareCode]` → View specific clip (e.g., `clips.peppi.app/ABC12345`)

### 2. Clip Viewer Page (`[shareCode]/+page.svelte`)

**Requirements:**
- Extract share code from URL parameter
- Fetch clip metadata from Supabase
- Display video player
- Show clip metadata (filename, date, duration, file size)
- Handle loading and error states
- Clean, minimal design

**API Endpoint:**
```typescript
// Query Supabase clips table
const { data: clip, error } = await supabase
  .from('clips')
  .select('*')
  .eq('share_code', shareCode.toUpperCase())
  .single();
```

**Clip Data Structure:**
```typescript
interface Clip {
  id: string;
  share_code: string;
  filename: string;
  b2_file_name: string;        // Path like "clips/ABC12345/video.mp4"
  file_size: number;
  duration_seconds: number | null;
  uploaded_at: string;
  metadata: any | null;        // May contain slippi_metadata
  user_id: string | null;
  device_id: string;
}
```

**Video URL Construction:**
- Use the Cloudflare Worker/CDN URL: `${CLIPS_CDN_URL}/${shareCode}/${filename}`
- Or construct B2 URL directly if CDN not available
- Example: `https://clips.peppi.app/ABC12345/Clip_Manual_20251115T160827_001.mp4`

### 3. Supabase Client Setup

```typescript
// src/lib/supabase.ts
import { createClient } from '@supabase/supabase-js';
import { PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY } from '$env/static/public';

export const supabase = createClient(
  PUBLIC_SUPABASE_URL,
  PUBLIC_SUPABASE_ANON_KEY
);
```

### 4. Component Structure

**Main Clip Viewer Component:**
```svelte
<script lang="ts">
  import { page } from '$app/stores';
  import { supabase } from '$lib/supabase';
  import { onMount } from 'svelte';
  
  let clip = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  
  const shareCode = $derived($page.params.shareCode?.toUpperCase());
  const videoUrl = $derived(
    clip?.b2_file_name 
      ? `${PUBLIC_CLIPS_CDN_URL}/${shareCode}/${clip.filename}`
      : null
  );
  
  onMount(async () => {
    if (!shareCode || shareCode.length !== 8) {
      error = 'Invalid share code';
      loading = false;
      return;
    }
    
    try {
      const { data, error: fetchError } = await supabase
        .from('clips')
        .select('*')
        .eq('share_code', shareCode)
        .single();
        
      if (fetchError) throw fetchError;
      clip = data;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Clip not found';
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div>Loading clip...</div>
{:else if error}
  <div>Error: {error}</div>
{:else if clip && videoUrl}
  <div>
    <h1>{clip.filename}</h1>
    <video controls src={videoUrl}></video>
    <div>
      <p>Share Code: {clip.share_code}</p>
      <p>Uploaded: {new Date(clip.uploaded_at).toLocaleString()}</p>
      {#if clip.duration_seconds}
        <p>Duration: {formatDuration(clip.duration_seconds)}</p>
      {/if}
      <p>Size: {formatBytes(clip.file_size)}</p>
    </div>
  </div>
{/if}
```

## Design Guidelines

- **Dark theme** (matches Peppi desktop app)
- **Minimal UI** - focus on the video
- **Responsive** - works on mobile and desktop
- **Fast loading** - optimize for quick clip viewing
- **Clean typography** - easy to read metadata

## Vercel Configuration

**vercel.json:**
```json
{
  "rewrites": [
    {
      "source": "/([A-Z0-9]{8})",
      "destination": "/$1"
    }
  ]
}
```

**Deployment:**
1. Connect GitHub repo to Vercel
2. Set environment variables in Vercel dashboard
3. Deploy
4. Configure custom domain `clips.peppi.app` in Vercel

## Dependencies

**package.json:**
```json
{
  "dependencies": {
    "@supabase/supabase-js": "^2.48.1",
    "svelte": "^5.28.1",
    "@sveltejs/kit": "^2.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5.0.3",
    "tailwindcss": "^4.1.7",
    "vite": "^6.3.5"
  }
}
```

## Key Implementation Notes

1. **Share Code Validation**: Must be exactly 8 characters, alphanumeric (uppercase)
2. **Error Handling**: Show friendly error messages for:
   - Invalid share code format
   - Clip not found
   - Network errors
   - Video loading failures

3. **Video Player**:
   - Use native HTML5 `<video>` with controls
   - Or integrate Plyr.js for better UX
   - Support fullscreen
   - Handle video loading states

4. **Metadata Display**:
   - Show filename, share code, upload date
   - Format file size (MB/GB)
   - Format duration (MM:SS)
   - Optionally show Slippi metadata if available

5. **SEO**:
   - Set proper `<title>` tag with share code
   - Add Open Graph meta tags
   - Add description meta tag

6. **Performance**:
   - Lazy load video
   - Optimize images/assets
   - Use Vercel's edge caching

## Example Routes

- `clips.peppi.app/ABC12345` → View clip with share code ABC12345
- `clips.peppi.app/XYZ98765` → View clip with share code XYZ98765

## Testing Checklist

- [ ] Valid 8-character share code displays clip
- [ ] Invalid share code shows error
- [ ] Video plays correctly
- [ ] Metadata displays correctly
- [ ] Responsive on mobile
- [ ] Fast loading times
- [ ] Error states handled gracefully
- [ ] Works with different video formats/sizes

## Additional Features (Optional)

- Share button (copy link to clipboard)
- Download button (if allowed)
- Embed code generation
- Related clips (if implementing a home page)
- Analytics tracking

## Reference

- Supabase docs: https://supabase.com/docs
- SvelteKit docs: https://kit.svelte.dev
- Vercel docs: https://vercel.com/docs
- Existing clip viewer component: `src/lib/components/clips/PublicClipViewer.svelte`

