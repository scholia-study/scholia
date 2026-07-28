import { createFileRoute } from "@tanstack/react-router";
import { ReviewPage } from "../modules/review";

// Access is enforced by the API: the payload 404s unless the caller is
// the article's author or holds `articles_review`, so this route needs
// no layout gate beyond authentication.
export const Route = createFileRoute("/_auth/articles/review/$requestId")({
    component: ReviewRoute,
});

function ReviewRoute() {
    const { requestId } = Route.useParams();
    return <ReviewPage requestId={requestId} />;
}
