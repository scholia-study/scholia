import { useState } from "react";
import { useGetNode } from "../api/nodes/nodes";
import type { SentenceResponse } from "../api/model";
import { Block } from "./BlockRenderer";

export function NodeContent({ slug, ncxId }: { slug: string; ncxId: string }) {
  const { data, isLoading, error } = useGetNode(slug, ncxId);
  const node = data?.data;
  const [selectedSentenceId, setSelectedSentenceId] = useState<string | null>(
    null,
  );

  const handleSelectSentence = (sentence: SentenceResponse) => {
    const next = sentence.id === selectedSentenceId ? null : sentence.id;
    setSelectedSentenceId(next);
    if (next) {
      console.log("Selected sentence:", sentence);
    }
  };

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
      <h1 className="text-2xl font-bold mb-8 text-stone-900">{node.label}</h1>
      {node.blocks.map((block) => (
        <Block
          key={block.id}
          block={block}
          selectedSentenceId={selectedSentenceId}
          onSelectSentence={handleSelectSentence}
        />
      ))}
    </article>
  );
}
