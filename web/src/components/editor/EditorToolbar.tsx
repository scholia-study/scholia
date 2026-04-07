import CodeOutlined from "@mui/icons-material/CodeOutlined";
import FormatBoldOutlined from "@mui/icons-material/FormatBoldOutlined";
import FormatItalicOutlined from "@mui/icons-material/FormatItalicOutlined";
import FormatListBulletedOutlined from "@mui/icons-material/FormatListBulletedOutlined";
import FormatListNumberedOutlined from "@mui/icons-material/FormatListNumberedOutlined";
import FormatQuoteOutlined from "@mui/icons-material/FormatQuoteOutlined";
import HorizontalRuleOutlined from "@mui/icons-material/HorizontalRuleOutlined";
import TitleOutlined from "@mui/icons-material/TitleOutlined";
import { IconButton, Menu, MenuItem, Tooltip, Typography } from "@mui/material";
import {
    toggleEmphasisCommand,
    toggleStrongCommand,
    toggleInlineCodeCommand,
    wrapInBlockquoteCommand,
    wrapInBulletListCommand,
    wrapInOrderedListCommand,
    wrapInHeadingCommand,
    insertHrCommand,
} from "@milkdown/kit/preset/commonmark";
import { editorViewCtx } from "@milkdown/kit/core";
import { commandsCtx } from "@milkdown/kit/core";
import { useInstance } from "@milkdown/react";
import { useState } from "react";

export function EditorToolbar() {
    const [loading, getInstance] = useInstance();
    const [headingAnchor, setHeadingAnchor] = useState<null | HTMLElement>(null);

    const run = (commandKey: unknown, payload?: unknown) => {
        if (loading) return;
        const editor = getInstance();
        editor.action((ctx) => {
            ctx.get(commandsCtx).call(commandKey as never, payload);
            ctx.get(editorViewCtx).focus();
        });
    };

    return (
        <div className="flex items-center gap-0.5 px-2 py-1 border-b border-stone-200 bg-stone-50">
            <Tooltip title="Bold">
                <IconButton size="small" onClick={() => run(toggleStrongCommand.key)}>
                    <FormatBoldOutlined fontSize="small" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Italic">
                <IconButton size="small" onClick={() => run(toggleEmphasisCommand.key)}>
                    <FormatItalicOutlined fontSize="small" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Inline code">
                <IconButton size="small" onClick={() => run(toggleInlineCodeCommand.key)}>
                    <CodeOutlined fontSize="small" />
                </IconButton>
            </Tooltip>

            <div className="w-px h-5 bg-stone-200 mx-1" />

            <Tooltip title="Heading">
                <IconButton size="small" onClick={(e) => setHeadingAnchor(e.currentTarget)}>
                    <TitleOutlined fontSize="small" />
                </IconButton>
            </Tooltip>
            <Menu
                anchorEl={headingAnchor}
                open={Boolean(headingAnchor)}
                onClose={() => setHeadingAnchor(null)}
            >
                <MenuItem
                    onClick={() => {
                        run(wrapInHeadingCommand.key, 0);
                        setHeadingAnchor(null);
                    }}
                >
                    <Typography variant="body2">Normal text</Typography>
                </MenuItem>
                {[1, 2, 3, 4, 5, 6].map((level) => (
                    <MenuItem
                        key={level}
                        onClick={() => {
                            run(wrapInHeadingCommand.key, level);
                            setHeadingAnchor(null);
                        }}
                    >
                        <Typography
                            variant="body2"
                            sx={{ fontWeight: 700, fontSize: `${1.25 - level * 0.1}rem` }}
                        >
                            H{level}
                        </Typography>
                    </MenuItem>
                ))}
            </Menu>
            <Tooltip title="Blockquote">
                <IconButton size="small" onClick={() => run(wrapInBlockquoteCommand.key)}>
                    <FormatQuoteOutlined fontSize="small" />
                </IconButton>
            </Tooltip>

            <div className="w-px h-5 bg-stone-200 mx-1" />

            <Tooltip title="Bullet list">
                <IconButton size="small" onClick={() => run(wrapInBulletListCommand.key)}>
                    <FormatListBulletedOutlined fontSize="small" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Numbered list">
                <IconButton size="small" onClick={() => run(wrapInOrderedListCommand.key)}>
                    <FormatListNumberedOutlined fontSize="small" />
                </IconButton>
            </Tooltip>

            <div className="w-px h-5 bg-stone-200 mx-1" />

            <Tooltip title="Horizontal rule">
                <IconButton size="small" onClick={() => run(insertHrCommand.key)}>
                    <HorizontalRuleOutlined fontSize="small" />
                </IconButton>
            </Tooltip>
        </div>
    );
}
