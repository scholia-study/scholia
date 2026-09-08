export const ACCEPTED_IMAGE_TYPES = ["image/jpeg", "image/png", "image/webp"];

export const MAX_UPLOAD_BYTES = 5 * 1024 * 1024;

/** First accepted image file in a drop or paste payload, if any. */
export function imageFileFrom(dt: DataTransfer | null): File | null {
    if (!dt) return null;
    for (const item of dt.items) {
        if (item.kind === "file" && ACCEPTED_IMAGE_TYPES.includes(item.type)) {
            return item.getAsFile();
        }
    }
    return null;
}
