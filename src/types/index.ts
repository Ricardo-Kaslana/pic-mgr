export interface Workspace {
  id: number
  name: string
  path: string
  created_at: string
  updated_at: string
}

export interface ImageRecord {
  id: number
  workspace_id: number
  relative_path: string
  file_name: string
  file_size: number | null
  file_hash: string | null
  width: number | null
  height: number | null
  format: string | null
  thumbnail_path: string | null
  notes: string | null
  rating: number | null
  created_at: string | null
  file_created_at: string | null
  updated_at: string
  indexed_at: string
  last_viewed_at: string | null
}

/// Lightweight image projection returned by `get_images_light`.
/// The grid only renders: thumbnail, file name, rating, selection state.
/// The context menu needs relative_path for "open in explorer" / "copy
/// path" / move / trash actions, so we include it (the rest of the heavy
/// fields — file_hash, notes, last_viewed_at, file_created_at — are still
/// excluded). A typical row is ~3x smaller than the full ImageRecord.
export interface ImageSummary {
  id: number
  workspace_id: number
  relative_path: string
  file_name: string
  file_size: number | null
  width: number | null
  height: number | null
  format: string | null
  thumbnail_path: string | null
  rating: number | null
  updated_at: string
  indexed_at: string
  created_at: string | null
}

export interface ImageDetail extends ImageRecord {
  tags: Tag[]
}

export interface Tag {
  id: number
  name: string
  color: string | null
}

export interface TagWithCount extends Tag {
  image_count: number
}

export interface ImagePage {
  images: ImageRecord[]
  total: number
  page: number
  page_size: number
}

export interface ImageSummaryPage {
  images: ImageSummary[]
  total: number
  page: number
  page_size: number
}

export interface ImageFilters {
  tag_ids: number[] | null
  format: string | null
  rating_min: number | null
  date_from: string | null
  date_to: string | null
  sort_by: string | null
  sort_dir: string | null
}

export interface ScanProgress {
  total: number
  processed: number
  new_files: number
  modified_files: number
  finished: boolean
}

export interface ThumbnailProgress {
  total: number
  generated: number
  failed: number
  finished: boolean
}