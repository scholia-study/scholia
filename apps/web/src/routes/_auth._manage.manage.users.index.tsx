import {
    Box,
    Checkbox,
    Chip,
    ListItemText,
    MenuItem,
    Select,
    type SelectChangeEvent,
    TextField,
} from "@mui/material";
import {
    DataGrid,
    type GridColDef,
    type GridPaginationModel,
    type GridRenderCellParams,
    type GridRenderEditCellParams,
    useGridApiContext,
    useGridApiRef,
} from "@mui/x-data-grid";
import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, notFound } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import {
    getListUsersQueryKey,
    useListUsers,
    useSetUserRoles,
} from "../api/admin-users/admin-users";
import { getMeQueryOptions } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import type { AdminUserRow } from "../api/model";
import { MANAGEABLE_ROLES, PAID_ROLES } from "../constants";
import { useDebouncedValue } from "../hooks/useDebouncedValue";

export const Route = createFileRoute("/_auth/_manage/manage/users/")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        if (!me?.data?.permissions?.includes("admin_panel")) {
            throw notFound();
        }
    },
    component: UsersDashboard,
});

const PAID: readonly string[] = PAID_ROLES;
const MANAGEABLE: readonly string[] = MANAGEABLE_ROLES;

const ROLE_LABELS: Record<string, string> = {
    admin: "Admin",
    editor: "Editor",
    scholiast: "Scholiast",
    honorary: "Honorary",
    scholiast_benefactor: "Scholiast (benefactor)",
    scholiast_patron: "Scholiast (patron)",
};
const label = (role: string) => ROLE_LABELS[role] ?? role;

const manageableOf = (roles: string[]) =>
    roles.filter((r) => MANAGEABLE.includes(r));
const sameSet = (a: string[], b: string[]) => {
    if (a.length !== b.length) return false;
    const sa = [...a].sort();
    const sb = [...b].sort();
    return sa.every((v, i) => v === sb[i]);
};

/// Multi-select of the manageable roles, opened in-place when the roles
/// cell enters edit mode. Paid tiers aren't listed (they're read-only).
function RolesEditCell(props: GridRenderEditCellParams<AdminUserRow>) {
    const { id, field, row } = props;
    const apiRef = useGridApiContext();
    const [value, setValue] = useState<string[]>(manageableOf(row.roles));

    const handleChange = (e: SelectChangeEvent<string[]>) => {
        const next = e.target.value;
        setValue(typeof next === "string" ? next.split(",") : next);
    };

    const commit = async () => {
        await apiRef.current.setEditCellValue({ id, field, value });
        apiRef.current.stopCellEditMode({ id, field });
    };

    return (
        <Select
            multiple
            open
            autoFocus
            value={value}
            onChange={handleChange}
            onClose={commit}
            renderValue={(sel) => sel.map(label).join(", ")}
            size="small"
            variant="standard"
            sx={{ width: "100%", fontSize: "0.75rem" }}
        >
            {MANAGEABLE_ROLES.map((role) => (
                <MenuItem key={role} value={role} dense>
                    <Checkbox
                        checked={value.includes(role)}
                        size="small"
                        sx={{ py: 0.25 }}
                    />
                    <ListItemText
                        primary={label(role)}
                        slotProps={{ primary: { sx: { fontSize: "0.8rem" } } }}
                    />
                </MenuItem>
            ))}
        </Select>
    );
}

function UsersDashboard() {
    const queryClient = useQueryClient();
    const apiRef = useGridApiRef();
    const setMutation = useSetUserRoles();

    const [searchInput, setSearchInput] = useState("");
    const search = useDebouncedValue(searchInput.trim());
    const [pagination, setPagination] = useState<GridPaginationModel>({
        page: 0,
        pageSize: 25,
    });

    const { data, isFetching } = useListUsers(
        {
            search: search || undefined,
            page: pagination.page + 1,
            per_page: pagination.pageSize,
        },
        { query: { placeholderData: keepPreviousData } },
    );

    const rows = data?.data?.users ?? [];
    const total = data?.data?.total ?? 0;

    // Persist a role change made in the edit cell. `newRow.roles` holds the
    // manageable selection; the backend preserves paid + `user` roles and
    // returns the full set, which we hand back to the grid.
    const processRowUpdate = async (
        newRow: AdminUserRow,
        oldRow: AdminUserRow,
    ) => {
        const desired = manageableOf(newRow.roles);
        if (sameSet(desired, manageableOf(oldRow.roles))) return oldRow;

        const res = await setMutation.mutateAsync({
            id: newRow.id,
            data: { roles: desired },
        });
        toast.success("Roles updated.");
        queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
        return res.data;
    };

    const columns = useMemo<GridColDef<AdminUserRow>[]>(
        () => [
            { field: "email", headerName: "Email", flex: 1.4, minWidth: 200 },
            {
                field: "display_name",
                headerName: "Name",
                flex: 1,
                minWidth: 140,
            },
            {
                field: "handle",
                headerName: "Handle",
                flex: 0.8,
                minWidth: 120,
                valueGetter: (value) => value ?? "—",
            },
            {
                field: "roles",
                headerName: "Roles",
                flex: 1.6,
                minWidth: 240,
                sortable: false,
                editable: true,
                renderCell: (params: GridRenderCellParams<AdminUserRow>) => (
                    <Box
                        sx={{
                            display: "flex",
                            gap: 0.5,
                            flexWrap: "wrap",
                            alignItems: "center",
                            height: "100%",
                        }}
                    >
                        {params.row.roles.map((r) => (
                            <Chip
                                key={r}
                                label={label(r)}
                                size="small"
                                variant={
                                    PAID.includes(r) ? "filled" : "outlined"
                                }
                                sx={{ height: 20, fontSize: "0.65rem" }}
                            />
                        ))}
                    </Box>
                ),
                renderEditCell: (params) => <RolesEditCell {...params} />,
            },
            {
                field: "email_verified",
                headerName: "Verified",
                width: 90,
                type: "boolean",
            },
            {
                field: "created_at",
                headerName: "Joined",
                width: 120,
                valueFormatter: (value: string) =>
                    new Date(value).toLocaleDateString(undefined, {
                        month: "short",
                        day: "numeric",
                        year: "numeric",
                    }),
            },
        ],
        [],
    );

    return (
        <div className="w-full max-w-6xl mx-auto px-8 py-12">
            <h1 className="text-2xl font-bold text-stone-900 mb-1">Users</h1>
            <p className="text-sm text-stone-500 mb-6">
                Manage user roles and access. Click a user's roles to edit.
            </p>

            <TextField
                value={searchInput}
                onChange={(e) => {
                    setSearchInput(e.target.value);
                    setPagination((p) => ({ ...p, page: 0 }));
                }}
                placeholder="Search email, name, or handle…"
                size="small"
                fullWidth
                sx={{ mb: 2, maxWidth: 360 }}
            />

            <Box sx={{ width: "100%" }}>
                <DataGrid<AdminUserRow>
                    apiRef={apiRef}
                    rows={rows}
                    columns={columns}
                    loading={isFetching}
                    rowCount={total}
                    paginationMode="server"
                    paginationModel={pagination}
                    onPaginationModelChange={setPagination}
                    pageSizeOptions={[25, 50, 100]}
                    disableColumnFilter
                    disableRowSelectionOnClick
                    editMode="cell"
                    processRowUpdate={processRowUpdate}
                    onProcessRowUpdateError={(err: unknown) => {
                        const message =
                            err instanceof FetchError && err.message
                                ? err.message
                                : "Failed to update roles";
                        toast.error(message);
                    }}
                    onCellClick={(params) => {
                        if (
                            params.field === "roles" &&
                            apiRef.current?.getCellMode(params.id, "roles") ===
                                "view"
                        ) {
                            apiRef.current.startCellEditMode({
                                id: params.id,
                                field: "roles",
                            });
                        }
                    }}
                    sx={{
                        border: "1px solid rgb(214 211 209)",
                        '& .MuiDataGrid-cell[data-field="roles"]': {
                            cursor: "pointer",
                        },
                    }}
                />
            </Box>
        </div>
    );
}
