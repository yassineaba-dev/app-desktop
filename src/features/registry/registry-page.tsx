import { useEffect, useRef, useState } from "react";
import { Inbox, ClipboardList, Download, LogOut, Settings, FileSpreadsheet } from "lucide-react";
import { useAuthStore } from "@/stores/auth-store";
import { useIncoming, useOutgoing } from "@/hooks/use-database";
import { databaseCommands } from "@/db/commands";
import { IncomingTable } from "../incoming/incoming-table";
import { IncomingForm } from "../incoming/incoming-form";
import { CorrespondenceTable } from "../correspondence/correspondence-table";
import { CorrespondenceForm } from "../correspondence/correspondence-form";
import { SettingsPage } from "../settings/settings-page";
import { ExcelImport } from "../import/excel-import";
import { useSyncRealtime } from "@/hooks/use-sync-realtime";
import type { Incoming, Outgoing } from "@/db/types";

const SPLIT_KEY = "registry-incoming-ratio";

function loadSplit(): number {
  const saved = Number(localStorage.getItem(SPLIT_KEY));
  return Number.isFinite(saved) && saved >= 0.2 && saved <= 0.8 ? saved : 0.5;
}

export function RegistryPage() {
  const clearAuth = useAuthStore((s) => s.clearAuth);
  const [showSettings, setShowSettings] = useState(false);

  useSyncRealtime();

  const [incPage, setIncPage] = useState(1);
  const [incSearch, setIncSearch] = useState("");
  const [incDateFilter, setIncDateFilter] = useState("");
  const [editIncoming, setEditIncoming] = useState<Incoming | null>(null);
  const incoming = useIncoming(incPage, incSearch || undefined, incDateFilter || undefined);

  const [outPage, setOutPage] = useState(1);
  const [outSearch, setOutSearch] = useState("");
  const [outDateFilter, setOutDateFilter] = useState("");
  const [editOutgoing, setEditOutgoing] = useState<Outgoing | null>(null);
  const outgoing = useOutgoing(outPage, outSearch || undefined, outDateFilter || undefined);

  const [downloading, setDownloading] = useState(false);
  const [showExcelImport, setShowExcelImport] = useState(false);

  const incFiltered = incSearch.trim().length > 0 || incDateFilter.trim().length > 0;
  const outFiltered = outSearch.trim().length > 0 || outDateFilter.trim().length > 0;

  const downloadDb = async (which: "incoming" | "outgoing" | "both") => {
    if (downloading) return;
    setDownloading(true);
    try {
      await databaseCommands.downloadFilteredDb({
        incoming: which === "incoming" || which === "both",
        outgoing: which === "outgoing" || which === "both",
        incomingSearch: which === "incoming" || which === "both" ? (incSearch || null) : null,
        incomingDate: which === "incoming" || which === "both" ? (incDateFilter || null) : null,
        outgoingSearch: which === "outgoing" || which === "both" ? (outSearch || null) : null,
        outgoingDate: which === "outgoing" || which === "both" ? (outDateFilter || null) : null,
      });
    } catch (err) {
      alert(String(err));
    } finally {
      setDownloading(false);
    }
  };

  const containerRef = useRef<HTMLDivElement | null>(null);
  const [incomingRatio, setIncomingRatio] = useState(loadSplit);

  useEffect(() => {
    localStorage.setItem(SPLIT_KEY, String(incomingRatio));
  }, [incomingRatio]);

  const handleResizeStart = (e: React.PointerEvent) => {
    e.preventDefault();
    const container = containerRef.current;
    if (!container) return;

    const onMove = (ev: PointerEvent) => {
      const rect = container.getBoundingClientRect();
      if (rect.width === 0) return;
      // The incoming section is anchored to the right side (RTL), so its
      // width is the distance from the left edge of the container to the
      // divider. This makes the divider follow the mouse exactly.
      const ratio = (rect.right - ev.clientX) / rect.width;
      setIncomingRatio(Math.min(0.8, Math.max(0.2, ratio)));
    };
    const onStop = () => {
      document.body.style.userSelect = "";
      container.removeEventListener("pointermove", onMove);
      container.removeEventListener("pointerup", onStop);
      container.removeEventListener("pointercancel", onStop);
    };
    document.body.style.userSelect = "none";
    container.addEventListener("pointermove", onMove);
    container.addEventListener("pointerup", onStop);
    container.addEventListener("pointercancel", onStop);
  };

  if (showSettings) {
    return <SettingsPage onBack={() => setShowSettings(false)} />;
  }

  return (
    <div className="flex flex-col h-full bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-slate-800 flex items-center justify-center">
            <Inbox className="w-5 h-5 text-white" />
          </div>
          <h1 className="text-lg font-bold text-slate-900">
            سجل الواردات والصادرات
          </h1>
        </div>

        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowExcelImport(true)}
            className="p-2 text-emerald-600 hover:text-emerald-700 hover:bg-emerald-50 rounded-lg transition-colors"
            title="استيراد من Excel"
          >
            <FileSpreadsheet className="w-5 h-5" />
          </button>
          {(incFiltered && outFiltered && (incoming.data?.total ?? 0) > 0 && (outgoing.data?.total ?? 0) > 0) && (
            <button
              onClick={() => downloadDb("both")}
              disabled={downloading}
              className="relative p-2 rounded-lg transition-colors disabled:opacity-60"
              style={{ color: "#7c3aed", backgroundColor: "#f3e8ff" }}
              title="تنزيل الواردات والصادرات معاً في قاعدة واحدة (حسب التصفية النشطة)"
            >
              <Download className="w-5 h-5" />
              <span className="absolute -top-0.5 -left-0.5 w-2.5 h-2.5 rounded-full bg-violet-500" />
            </button>
          )}
          <button
            onClick={() => setShowSettings(true)}
            className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition-colors"
            title="الإعدادات"
          >
            <Settings className="w-5 h-5" />
          </button>
          <button
            onClick={clearAuth}
            className="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
            title="تسجيل الخروج"
          >
            <LogOut className="w-5 h-5" />
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-hidden">
        <div ref={containerRef} className="flex h-full">
          <section
            className="min-w-0 bg-white border-b border-gray-200 flex flex-col shrink-0"
            style={{ width: `${incomingRatio * 100}%` }}
          >
            <div className="flex items-center gap-2.5 border-b border-gray-100 px-4 py-2 shrink-0 bg-white">
              <div className="w-7 h-7 rounded-md bg-blue-100 flex items-center justify-center">
                <Inbox className="w-4 h-4 text-blue-600" />
              </div>
              <h2 className="text-base font-bold text-slate-900">الواردات</h2>
              {incFiltered && (incoming.data?.total ?? 0) > 0 ? (
                <button
                  onClick={() => downloadDb("incoming")}
                  disabled={downloading}
                  className="ms-auto flex items-center gap-1.5 px-3 py-1.5 text-sm font-semibold text-white bg-blue-600 hover:bg-blue-700 rounded-md shadow-sm transition-colors disabled:opacity-60"
                  title="تنزيل بيانات الواردات المصفّاة"
                >
                  <Download className="w-4 h-4" />
                  تنزيل المفلتر
                  <span className="text-xs font-medium text-blue-100">
                    ({incoming.data?.total ?? 0})
                  </span>
                </button>
              ) : null}
            </div>
            <div className="flex-1 min-h-0 flex flex-col">
              <IncomingTable
                items={incoming.data?.items ?? []}
                total={incoming.data?.total ?? 0}
                page={incPage}
                perPage={incoming.data?.per_page ?? 20}
                isLoading={incoming.isLoading}
                search={incSearch}
                onSearchChange={setIncSearch}
                dateFilter={incDateFilter}
                onDateFilterChange={setIncDateFilter}
                onPageChange={setIncPage}
                onEdit={setEditIncoming}
              />
            </div>
          </section>

          <div
            onPointerDown={handleResizeStart}
            title="اسحب لتغيير عرض الجداول"
            className="group relative z-10 w-2 shrink-0 cursor-col-resize touch-none select-none bg-gray-200 hover:bg-brand-300/70 active:bg-brand-500 transition-colors"
          >
            <div className="absolute inset-y-0 left-1/2 w-[2px] -translate-x-1/2 bg-gray-300 group-hover:bg-white group-active:bg-white" />
          </div>

          <section className="flex-1 min-w-0 bg-white border-b border-gray-200 flex flex-col">
            <div className="flex items-center gap-2.5 border-b border-gray-100 px-4 py-2 shrink-0 bg-white">
              <div className="w-7 h-7 rounded-md bg-violet-100 flex items-center justify-center">
                <ClipboardList className="w-4 h-4 text-violet-600" />
              </div>
              <h2 className="text-base font-bold text-slate-900">الصادرات</h2>
              {outFiltered && (outgoing.data?.total ?? 0) > 0 ? (
                <button
                  onClick={() => downloadDb("outgoing")}
                  disabled={downloading}
                  className="ms-auto flex items-center gap-1.5 px-3 py-1.5 text-sm font-semibold text-white bg-violet-500 hover:bg-violet-600 rounded-md shadow-sm transition-colors disabled:opacity-60"
                  title="تنزيل بيانات الصادرات المصفّاة"
                >
                  <Download className="w-4 h-4" />
                  تنزيل المفلتر
                  <span className="text-xs font-medium text-violet-100">
                    ({outgoing.data?.total ?? 0})
                  </span>
                </button>
              ) : null}
            </div>
            <div className="flex-1 min-h-0 flex flex-col">
              <CorrespondenceTable
                items={outgoing.data?.items ?? []}
                total={outgoing.data?.total ?? 0}
                page={outPage}
                perPage={outgoing.data?.per_page ?? 20}
                isLoading={outgoing.isLoading}
                search={outSearch}
                onSearchChange={setOutSearch}
                dateFilter={outDateFilter}
                onDateFilterChange={setOutDateFilter}
                onPageChange={setOutPage}
                onEdit={setEditOutgoing}
              />
            </div>
          </section>
        </div>
      </div>

      {editIncoming && (
        <IncomingForm editItem={editIncoming} onClose={() => setEditIncoming(null)} />
      )}
      {editOutgoing && (
        <CorrespondenceForm editItem={editOutgoing} onClose={() => setEditOutgoing(null)} />
      )}
      {showExcelImport && <ExcelImport onClose={() => setShowExcelImport(false)} />}
    </div>
  );
}
