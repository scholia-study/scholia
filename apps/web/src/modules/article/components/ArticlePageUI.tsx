export function ArticlePageUI({ kind }: { kind: "loading" | "error" }) {
    switch (kind) {
        case "error": {
            return (
                <div className="flex-1 bg-white">
                    <div className="max-w-3xl mx-auto px-8 py-16">
                        <p className="text-sm text-red-500">
                            Something went wrong or the article wasn't found.
                        </p>
                    </div>
                </div>
            );
        }
        case "loading": {
            return (
                <div className="flex-1 bg-white">
                    <div className="max-w-3xl mx-auto px-8 py-16">
                        <p className="text-sm text-stone-400">Loading...</p>
                    </div>
                </div>
            );
        }
    }
}
