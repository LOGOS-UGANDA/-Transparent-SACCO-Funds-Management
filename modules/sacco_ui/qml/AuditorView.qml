import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Auditor view: full trail with evidence links (issue #5 checklist).
Item {
    id: root
    property var bridge
    property var trail: []
    property string message: ""

    function refresh() {
        if (!bridge) return
        try {
            const response = JSON.parse(bridge.call("getAuditTrail", []))
            trail = response.data && Array.isArray(response.data) ? response.data : []
            message = trail.length
                ? "Showing indexed transaction records; the current indexer response does not include decodable shard data."
                : "No indexed transactions are available. Check indexer sync status."
        } catch (e) { message = "sacco_core not reachable: " + e }
    }
    Component.onCompleted: refresh()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        Label { text: "Audit trail"; font.bold: true; font.pixelSize: 18 }
        Label { text: root.message; color: "firebrick"; visible: root.message !== "" }
        Button { text: "Refresh"; onClicked: root.refresh() }

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            model: root.trail
            delegate: Frame {
                width: ListView.view.width
                ColumnLayout {
                    width: parent.width
                    Label { text: JSON.stringify(modelData); wrapMode: Text.Wrap; Layout.fillWidth: true }
                }
            }
        }
    }
}
