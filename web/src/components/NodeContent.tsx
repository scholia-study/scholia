import { useGetNode } from "../api/nodes/nodes";
import { Block } from "./BlockRenderer";
import { useSentenceSelection } from "./SentenceSelectionContext";

export function NodeContent({ slug, nodeSlug }: { slug: string; nodeSlug: string }) {
  const { data, isLoading, error } = useGetNode(slug, nodeSlug);
  const node = data?.data;
  const { selectedSentenceId, onSelectSentence } = useSentenceSelection();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full text-stone-400">
        <p>Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full text-red-500">
        <p>Failed to load content.</p>
      </div>
    );
  }

  if (!node) return null;

  return (
    <article className="max-w-2xl mx-auto px-8 py-12">
      {node.blocks.map((block) => (
        <Block
          key={block.id}
          block={block}
          selectedSentenceId={selectedSentenceId}
          onSelectSentence={onSelectSentence}
        />
      ))}
    </article>
  );
}
