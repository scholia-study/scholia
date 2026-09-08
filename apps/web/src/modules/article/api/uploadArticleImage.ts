import customFetch from "../../../api/fetcher";
import type { UploadedImageResponse } from "../../../api/model";

/** Hand-written multipart upload. The orval-generated hook can't be used
 *  here: it would set a Content-Type without the multipart boundary. With
 *  a FormData body and no explicit header, the browser sets both. */
export async function uploadArticleImage(
    file: File,
): Promise<UploadedImageResponse> {
    const form = new FormData();
    form.append("file", file, file.name);
    const res = await customFetch<{
        data: UploadedImageResponse;
        status: number;
    }>("/api/user/article-images", {
        method: "POST",
        body: form,
    });
    return res.data;
}
