"use strict";

define(["knockout", "reqwest", "handlers"], function(ko, reqwest, _handlers) {
	const Application = function() {
		this.loading = ko.observable(false);
		this.selectedConnection = ko.observable(undefined);
		this.availableConnections = ko.observableArray([]);
		this.queryRegionHierarchy = ko.observable("");
		this.preferredLanguage = ko.observable("");
		this.availableLanguages = ko.observableArray([]);
		this.showBiggerRegions = ko.observable(false);
		this.regionNames = ko.observable({});
		this.regionHierarchies = ko.observableArray([]);
		this.errorMessage = ko.observable("");

		this.isConnectionInvalid = ko.pureComputed(function() {
			return this.selectedConnection() === undefined;
		}, this);

		this.isRegionNameInvalid = ko.pureComputed(function() {
			return this.queryRegionHierarchy() === "";
		}, this);

		this.isFormInvalid = ko.pureComputed(function() {
			return this.isConnectionInvalid() || this.isRegionNameInvalid();
		}, this);

		this.isLoading = ko.pureComputed(function() {
			return this.loading();
		}, this);

		this.isHierarchyAvailable = ko.pureComputed(function() {
			return this.regionHierarchies().length > 0;
		}, this);

		this.isErrorMessagePresent = ko.pureComputed(function() {
			return this.errorMessage() !== "";
		}, this);

		this.regionHierarchiesFiltered = ko.pureComputed(function() {
			if (this.showBiggerRegions()) {
				return this.regionHierarchies();
			} else {
				return this.regionHierarchies().filter(hierarchy => !hierarchy.bigger);
			}
		}, this);

		this.updateConnections();
	};

	Application.prototype.updateConnections = function() {
		this.loading(true);

		reqwest({
			url: "/api/v1/connections",
			type: "json",
			method: "POST",
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.availableConnections(resp.result);
						this.selectedConnection(undefined);
						this.errorMessage("");
					} else {
						this.errorMessage(resp.message);
					}

					this.loading(false);
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.errorMessage(msg || err.responseText);
					this.loading(false);
				}.bind(this)
			);
	};

	Application.prototype.searchRegion = function() {
		this.loading(true);

		reqwest({
			url: "/api/v1/find_region",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({
				connection: this.selectedConnection(),
				query: this.queryRegionHierarchy(),
			}),
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.regionNames(resp.result.regions);
						this.regionHierarchies(resp.result.hierarchies);
						this.errorMessage("");

						this.updateLanguageList();
					} else {
						this.regionNames({});
						this.regionHierarchies([]);
						this.errorMessage(resp.message);
					}

					this.loading(false);
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.errorMessage(msg || err.responseText);
					this.loading(false);
				}.bind(this)
			);
	};

	Application.prototype.updateLanguageList = function() {
		const languages = new Set();

		Object.values(this.regionNames()).forEach(function(region) {
			Object.keys(region.names).forEach(function(name) {
				languages.add(name);
			});
		});

		this.availableLanguages(Array.from(languages).sort());
	};

	Application.prototype.namedHierarhy = function(hierarchy) {
		const partNames = [];
		const regionNames = this.regionNames();
		const preferredLanguage = this.preferredLanguage();

		for (const partId of hierarchy.parts) {
			if (partId in regionNames) {
				const region = regionNames[partId];
				const name = region.names[preferredLanguage] || region.default_name;

				partNames.push(name);
			} else {
				partNames.push(`<${partId}>`);
			}
		}

		return partNames.join(" > ");
	};

	Application.prototype.areaCode = function(hierarchy) {
		return `<Area adminPlaceID="${hierarchy.id}"/>`;
	};

	return Application;
});
